# Speech engine evaluation: final report (M8)

Project: AssistantCabinetAI-SpeechLab · Evidence cut-off: 2026-10-09 (repository at commit `1f9257c` plus the M8 changes) ·
Sections follow `Plan.md` section 12.

**Scope in one paragraph.** One laptop (Intel Core 7 150U, 12 logical cores, 23.6 GB RAM, Windows 11 Home x64, CPU only, no
accelerator), one speaker (the owner, standard metropolitan French, one microphone), 95 read sentences (9.05 minutes,
1,054 scored reference words: 803 French, 251 English), engines run on whole clips, not in streaming mode. Every figure below
comes from a run folder in `benchmark/results/` or from an entry of `docs/PROJECT_LOG.md`; nothing is simulated or
extrapolated. Claims are marked VERIFIED (observed here, evidence named) or NOT VERIFIED (with the steps to verify); the full
list is in Appendix A. The licensing part is a technical reading of public licence texts and model cards, **not legal advice**.

Conventions: WER = word error rate (micro: total errors over total reference words); RTF = processing time over audio
duration (below 1 is faster than real time); "interval" = 95 % bootstrap interval over sentences (`scripts/bootstrap_ci.py`,
seed 7, 5,000 resamples); `[C-nn]` points to the claims table.

---

## 1. Executive summary

**What the evidence supports**

1. **sherpa-onnx with the NeMo Parakeet TDT 0.6B v3 int8 model is the most accurate and the only interactive speech-to-text
   configuration on this CPU**: WER 2.3 % (interval 1.4 to 3.4), RTF about 0.10 to 0.11 (a 5-second sentence in about 0.5 s),
   no failure on any of the 95 sentences or on the long clips tried [C-01, C-04, C-10]. Its costs: 1.6 to 1.9 GB of memory,
   7 to 10 busy cores although 4 threads are requested, a cold load of about 2 s, and no way to cancel a decode in progress [C-06, C-07].
2. **whisper.cpp small (q5_1) is accurate (3.9 % with 5 beams, interval 2.5 to 5.4) but slower than real time on this CPU**
   (RTF 1.1 to 1.3, 5 to 6 s for a 4 to 5 s sentence), so it does not suit voice queries here. Its strengths are real
   cancellation, a small memory footprint (0.4 to 0.5 GB) and an MIT licence. Parakeet is better than it by 1.6 points
   (interval +0.1 to +3.1): borderline, not a large gap [C-02]. whisper.cpp tiny and base (8 to 20 % WER) and sherpa-onnx
   Whisper tiny (25 %) are too inaccurate; Canary int8 is comparable on clean sentences but emitted garbage twice and drops
   endings on long clips [C-08, C-09].
3. **Every engine misspells drug names** (best: 4 of 6 found). A strict dictionary post-correction fixed 2 more with no word
   broken, on an upper-bound test; engine-side vocabulary biasing was riskier (D-036) [C-17 to C-20]. A low WER does not
   make the output safe for clinical vocabulary.
4. **Text-to-speech**: Piper voices through sherpa-onnx run 13 to 29 times faster than real time and need 200 to 260 MB
   [C-21]. English is solved with licence-clean voices (`ljspeech`, `kristin` rated `clear`; `libritts_r` `attribution`). **French
   is not**: the two French voices the owner judges good (Piper siwis, gilles) have restricted parent checkpoints (rated `review`
   and `excluded`), and the only freely usable French voice (Coqui css10) scores 3 out of 5 at best [C-25, C-47]. The owner keeps
   siwis and gilles for development and will re-train them from a clean base before any commercial release (D-043).
5. **Licensing is the main obstacle, not engineering**: the full sherpa-onnx libraries contain the GPL-3.0 phonemizer espeak-ng
   (found in the packaged `speechlab.exe`: 95 occurrences) [C-46]; the NeMo models need CC BY 4.0 attribution; the
   French voice lineage is unresolved; the application has no credits view and no licence of its own (I-067, I-072).
6. **Windows is verified end to end** (per-user installer of 8.45 MiB, both engines through the packaged app, microphone in the
   release origin, offline proof for the application's own processes). **macOS is NOT VERIFIED** (no Mac): a checklist exists
   [C-34 to C-41].

**What is unresolved** (what would settle each item is in section 10): other speakers, noise, conversational speech and
accents (none measured; accents dropped by the owner, D-046); a weaker target machine; whisper.cpp large-v3-turbo and any
accelerated build (never run); streaming recognition; the legal reading of the GPL phonemizer and of the French voice
lineage; macOS; a clean Windows machine; signing.

**Recommendation in five lines (provisional, section 10).** (1) Keep both engines behind the existing provider contract. (2) Default
speech-to-text: sherpa-onnx Parakeet TDT v3 int8; keep whisper.cpp small as a second provider (cancellation, memory, MIT) until
the open points are measured. (3) No scenario split is supported by the evidence. (4) Text-to-speech: sherpa-onnx with Piper English
voices rated `clear`/`attribution`; French only after the re-trained voices or a legal clearance. (5) Do not ship any build with
speech synthesis before the espeak-ng question is decided.

### 1.1 The twelve questions of `Plan.md` section 2

| # | Question | Answer from the evidence | Status |
|---|---|---|---|
| 1 | Most reliable French STT engine | Parakeet: French WER 2.1 % (1.1 to 3.5), no failure. whisper.cpp small 4.2 % (2.5 to 6.2). One speaker. | answered for one speaker |
| 2 | Accents (French, Swiss French) | **No accent figure exists.** The owner has no clips and dropped the evaluation (D-046). | not answered, out of scope |
| 3 | Specialised vocabulary | Every engine misspells drug names (best 4 of 6); tech terms 31 of 34 at best; mitigation partial (section 4). | answered, upper bound |
| 4 | English reliability | Parakeet 2.8 %, whisper.cpp small 2.8 %; only 25 sentences (wide intervals). | answered, weak power |
| 5 | Offline operation | Proven for the application's own processes (section 8); whole-network-off test not done; the web view runtime keeps one outside connection. | partly verified |
| 6 | CPU, RAM, latency, disk | Section 6. | answered for one machine |
| 7 | Short queries versus dictation | Parakeet is the only interactive engine for queries and had no failure on long clips; whisper.cpp small suits neither interactive use nor long dictation on this CPU (RTF above 1). | answered, low power on long clips |
| 8 | Natural French and English TTS with sherpa-onnx | English yes (owner's scores 4 to 5 for the Piper voices); French good only with voices of restricted lineage. | answered (opinion + licence) |
| 9 | Male and female voices | Yes in capability: French female (siwis) and male (gilles, 16 kHz, `excluded`); English multi-speaker `libritts_r`; no clean good French male voice. | partly |
| 10 | Adjustable playback speed | Yes, passed to the library; works on both families, differently (Piper compressed, Kokoro linear). | verified |
| 11 | Integration without coupling | Yes: provider traits, both engines run end to end in the Tauri app (Windows). | verified (Windows) |
| 12 | Windows versus macOS | Windows verified; macOS NOT VERIFIED. | partly |

---

## 2. Engine comparison

### 2.1 Test conditions and runs used

| Run folder | Content | Reps | Decoding | Background CPU before the run |
|---|---|---|---|---|
| `20261007-215116-full-owner-reps1-clean` | 9 configurations x 95 sentences (accuracy and speed) | 1 | sherpa-onnx greedy; whisper.cpp 5 beams and greedy | 19.4 % (8 s average, AC power) |
| `20261007-201137-full-owner-reps1` | same, **disturbed** by a heavy task: accuracy only (transcripts identical to the clean run, 855 of 855); its speed figures are not quoted | 1 | same | not quiet |
| `20261008-035938-timing-3reps` | 9 configurations x 40 short sentences, spread of the same sample | 3 | same | 19.9 % before, 21.5 % after |
| `20261008-045952-long-whole`, `-050308-long-vad`, `-050621-short-vad`, `-054129-long-vad-max15`, `-054529-long-vad-max10` | long audio, whole versus cut at silences | 1 | same | 19.2 %, 28.4 %, 14.4 %, retried until under 30 % |
| `20261008-0653*` to `-0735*` (`*-t4-*`) | drug names and key terms | 1 | same | 11 to 19 % |

Common conditions (`system.json`, `config.json`): release build, 4 threads per engine, one child process per
configuration, `forcedDespitePreflight` = false, Windows power plan "balanced", AC power. The background load of about 20 % comes
from Windows services and the security software (D-034); it is **not an idle machine**. Speed figures carry the load of their run and
**a margin of 10 to 20 %**: inside one run the same sample varies by 1 to 4 % (median), but two runs of the same code path differed
by up to 20 % for the fast models and by 1 to 48 % in another comparison (I-012, I-042) [C-05]. Compare configurations inside one run
only.

whisper.cpp is v1.9.4 built from source, CPU only, run as an external `whisper-cli` process (a new process, so a cold start, for
every transcription). sherpa-onnx is v1.13.8 through its official Rust crate, statically linked, model kept loaded.

### 2.2 Accuracy and speed, clean full run (95 sentences, 1 repetition)

| Configuration | WER (interval) | Critical samples | RTF median | Inference median (ms) | p95 (ms) | Cold load (ms) | Peak memory (MB) | Busy cores |
|---|---|---|---|---|---|---|---|---|
| sherpa-onnx Parakeet TDT v3 int8, greedy | **2.3 %** (1.4 to 3.4) | 4 | 0.114 | 551 | 785 | 1,956 | 1,875 | 9.4 |
| whisper.cpp small q5_1, 5 beams | 3.9 % (2.5 to 5.4) | 7 | 1.236 | 5,881 | 6,837 | 199 | 533 | 3.9 |
| whisper.cpp small q5_1, greedy | 4.5 % (3.0 to 6.1) | 7 | 1.117 | 5,403 | 6,070 | 199 | 410 | 3.9 |
| sherpa-onnx Canary 180M flash int8, greedy | 5.9 % (2.0 to 13.1); 3.0 % without one garbage output | 3 | 0.111 | 536 | 903 | 1,147 | 1,087 | 7.4 |
| whisper.cpp base q5_1, 5 beams | 8.4 % (6.3 to 10.9) | 12 | 0.337 | 1,626 | 1,910 | 88 | 265 | 3.9 |
| whisper.cpp base q5_1, greedy | 11.4 % (8.8 to 14.2) | 15 | 0.293 | 1,426 | 1,626 | 88 | 216 | 3.8 |
| whisper.cpp tiny fp16, 5 beams | 16.1 % (12.8 to 19.9) | 13 | 0.159 | 762 | 915 | 104 | 242 | 3.7 |
| whisper.cpp tiny fp16, greedy | 19.8 % (15.8 to 23.9) | 18 | 0.129 | 625 | 674 | 103 | 213 | 3.5 |
| sherpa-onnx Whisper tiny, greedy | 25.4 % (19.8 to 31.9) | 19 | 0.077 | 365 | 578 | 490 | 894 | 7.4 |

Sources: `summary.md` of the clean run (accuracy, speed, memory) and `scripts/bootstrap_ci.py` (intervals), re-run for this report with
identical output. Peak memory is the maximum over samples, a sampled lower bound; for sherpa-onnx it covers the whole benchmark
process, for whisper.cpp the child process. "Critical samples" counts samples with at least one critical-severity flag (changed,
lost or added number, unit, negation, weekday/month, missing drug or name); the detector is a heuristic with roughly 80 to
85 % precision on a small hand review (I-034) [C-13].

Speed on the short sentences of the timing study (40 sentences, 3 repetitions, 1,080 runs, 0 failures, 0 changing
transcripts) [C-03, C-05]:

| Configuration | RTF median | Inference median / p95 (ms) | Median CV across repetitions | Peak memory median (MB) |
|---|---|---|---|---|
| Parakeet | 0.099 | 440 / 558 | 3.5 % | 1,578 |
| Canary | 0.095 | 414 / 528 | 2.4 % | 761 |
| sherpa-onnx Whisper tiny | 0.059 | 268 / 400 | 2.4 % | 669 |
| whisper.cpp small, 5 beams / greedy | 1.318 / 1.261 | 5,697 / 5,374 | 0.7 % / 1.4 % | 478 / 360 |
| whisper.cpp base, 5 beams / greedy | 0.313 / 0.293 | 1,356 / 1,272 | 1.2 % / 0.7 % | 212 / 167 |
| whisper.cpp tiny, 5 beams / greedy | 0.143 / 0.121 | 608 / 517 | 1.2 % / 0.9 % | 190 / 163 |

**How to quote the speed** (with the 10 to 20 % margin): Parakeet and Canary about 9 to 10 times faster than real time
(RTF 0.10 to 0.11); whisper.cpp base about 3 times faster (0.29 to 0.34); whisper.cpp small **slower than real time**
(1.1 to 1.3: its whole min to max range of 5.0 to 6.7 s on the timing sentences, about 4.4 s long, lies above the sentence duration).

### 2.3 What the data supports, and what it does not

- Parakeet has the lowest WER. Paired bootstrap against it: whisper.cpp small 5 beams +1.6 points (+0.1 to +3.1), small greedy
  +2.2 (+0.7 to +3.8), base, tiny and sherpa Whisper tiny clearly worse (all intervals above 0); Canary +3.6 (-0.3 to +10.5)
  with its garbage output and +0.8 (-0.6 to +2.3) without it: **not distinguishable** from Parakeet. Excluding the suspect sample
  `en-it-05-owner` (I-033) changes no conclusion (Parakeet 2.2 %, Canary 3.0 %, small 3.7 %).
- Inside Whisper, model size dominates: tiny 16 to 20 %, base 8 to 11 %, small 4 to 5 %; 5-beam search beats greedy by 0.6 to 3.3
  points at 5 to 30 % more time.
- The same Whisper tiny weights score 16.1 % in whisper.cpp (5 beams) and 25.4 % in sherpa-onnx (greedy, with repetition
  loops): the runtime and decoding matter, not only the weights (I-018, I-023).
- Accuracy does not depend on the machine load or the repetition: 855 of 855 transcripts identical between the disturbed and the
  clean run, 360 of 360 between the clean run and the timing study, 828 of 828 with chunking on for short clips [C-03].
- Memory and busy cores did not change with the load (within 1 %); they are properties of the engines. sherpa-onnx uses
  7.4 to 10.4 busy cores for 4 configured threads (cause unknown, I-036), whisper.cpp 3.5 to 3.9 [C-06].
- Not measured: `whisper-cpp-large-v3-turbo-q5_0` is in the manifest but was never installed or run; no GPU or accelerated
  build; no second machine; no streaming mode (I-037, I-074) [C-52, C-53].

### 2.4 Integration properties (verified in the Tauri application)

| Property | whisper.cpp (external `whisper-cli`) | sherpa-onnx (in-process, official Rust crate) |
|---|---|---|
| Licence of code | MIT | Apache-2.0 (runtime libraries include espeak-ng GPL-3.0, section 9) |
| How it runs | child process per transcription; model reloaded each time (0.09 to 0.2 s for tiny, base, small) | model loaded once (0.5 to 2 s), stays in memory |
| Cancellation | real: the process is killed (verified from the UI, no process left) | none during a decode (blocking call, `supportsCancellation` false, I-011) |
| Build | CMake, 91 s, from the pinned tag | prebuilt static libraries downloaded by the crate (an antivirus intercepting TLS breaks that download, I-009) |
| Packaging | sidecar `whisper-cli.exe` **plus four DLLs**; needs the Visual C++ runtime (I-070) | linked into `speechlab.exe` (31.4 MB) |
| Models verified | tiny, base, small | Whisper tiny, Canary, Parakeet |
| Long audio | no failure seen up to 45.8 s | Parakeet no failure; Canary and Whisper tiny failures (below) |

Long audio (M5e, D-035; 3 reference dictations of 26 to 36 s and 2 private clips, little statistical power, one dropped
sentence moves a WER by 20 points): only Parakeet and every whisper.cpp model had no failure. sherpa-onnx Whisper cannot take more than
30 s (the library discards the rest: last sentence lost on a 35.9 s dictation) and loops; Canary dropped the closing word of a 45.8 s
clip in both modes and returned half the words (in 63 s) on one 25 s piece [C-08, C-09, C-10]. Cutting at silences (Silero VAD)
leaves short audio provably unchanged, helps only sherpa Whisper tiny, is neutral for Parakeet and whisper.cpp and harmful for
Canary (WER 1.5 % to 6.5 % on three dictations); it stays OFF by default. The output guard for runaway text is not built (I-035).

### 2.5 Single-speaker limit

All accuracy figures are for one speaker reading sentences in a quiet room with one microphone. They do not say how the engines
behave with other voices, noise, conversational speech, different speaking speeds or accents. Sampling noise over sentences is
quantified by the intervals; speaker, microphone and recording variation is not [C-15, C-16].

---

## 3. French accuracy, English accuracy

Computed from the clean run with `scripts/wer_by_language.py` (added in M8; same method as `bootstrap_ci.py`, applied per
language). French = 70 sentences, 803 reference words (general 24, with English terms 20, medical 10, administrative 7, legal 7,
dictation 2); English = 25 sentences, 251 words (general 16, technical 8, dictation 1) [C-11].

| Configuration | French WER (interval) | French critical samples | English WER (interval) | English critical samples |
|---|---|---|---|---|
| Parakeet | **2.1 %** (1.1 to 3.5) | 3 | 2.8 % (1.0 to 5.3) | 1 |
| whisper.cpp small, 5 beams | 4.2 % (2.5 to 6.2) | 4 | **2.8 %** (1.0 to 5.3) | 3 |
| whisper.cpp small, greedy | 4.9 % (3.2 to 7.0) | 4 | 3.2 % (1.4 to 5.8) | 3 |
| Canary | 3.0 % (1.6 to 4.8) | 2 | 15.1 % (1.4 to 49.5), one garbage output | 1 |
| whisper.cpp base, 5 beams / greedy | 9.6 % (7.1 to 12.7) / 12.7 % (9.6 to 16.0) | 9 / 11 | 4.8 % (2.1 to 8.9) / 7.2 % (3.7 to 12.6) | 3 / 4 |
| whisper.cpp tiny, 5 beams / greedy | 19.2 % (15.5 to 23.5) / 24.0 % (19.6 to 28.6) | 11 / 17 | 6.4 % (2.7 to 12.1) / 6.4 % (3.0 to 11.5) | 2 / 1 |
| sherpa-onnx Whisper tiny | 25.0 % (20.6 to 29.6) | 16 | 26.7 % (9.5 to 53.5) | 3 |

Reading the table:

- **French**: Parakeet and Canary (on French) are the best; whisper.cpp small follows. The small Whisper models are clearly
  better on English than on French (tiny 6.4 % versus 19.2 %); the cause was not investigated.
- **English limits**: 25 sentences and 251 words, so the intervals are wide and the English ranking among the top four
  (Parakeet, small, base, tiny) is mostly noise. Canary's 15.1 % comes from one runaway output on a slow reading (WER 517 % on
  that sample, I-035); without it Canary is at the level of the others. The English set includes technical terms (PostgreSQL,
  API, firewall) which are the usual source of errors (45.9 % for Canary on `en-technical`, 5.4 % for Parakeet).
- **Categories** (WER micro, Parakeet / small 5 beams): general French 2.9 / 3.4 %, medical 1.8 / 6.3 %, administrative 2.7 / 1.4 %,
  legal 1.5 / 0.0 %, French with English terms 2.5 / 7.0 %, French dictation 0.7 / 3.4 %, English technical 5.4 / 5.4 %. A category holds
  7 to 24 sentences (only 2 French and 1 English dictation): do not rank on them.
- **Suspect sample** `en-it-05-owner` ("Open port 443 on the firewall."): all 9 configurations hear "543". The owner has not listened
  again (I-033); it is probably a reading problem, and excluding it changes no conclusion.
- **Critical errors exist in every configuration**, including the best: all engines misspell "amoxicilline" (Parakeet
  "amoxycilline", Canary "amoxiciline"), Parakeet turned "10 jours" into "1 jours" on one sentence. A WER of 2 % is compatible
  with changed numbers and drug names; the flags, not the WER, carry the safety information (D-026).
- **Accents**: **no accent figure exists.** `Plan.md` lists regional and Swiss French accents; the owner had no clips and
  dropped the evaluation (D-046). All figures are for standard metropolitan French from one speaker. The names "Lausanne" and
  "francs suisses" appear in sentences but say nothing about Swiss speech; Swiss number words (septante, huitante, nonante) are not
  handled by the scoring (I-030) [C-14].
- **Conditions not covered**: speaking speed variations, natural conversational speech, noisy rooms, other microphones, other speakers
  [C-15, C-16].

---

## 4. Specialised vocabulary

The dataset has 56 key-term occurrences (drug 6, legal 6, tech 34, general term 10) in the public sentences. Counted with
`scripts/wer_by_language.py` (same numbers as `bench termstudy`):

| Configuration | drug | legal | tech | term |
|---|---|---|---|---|
| Parakeet | 4/6 | 6/6 | 31/34 | 10/10 |
| Canary | 4/6 | 6/6 | 21/34 | 9/10 |
| whisper.cpp small 5 beams / greedy | 3/6 / 3/6 | 6/6 / 6/6 | 28/34 / 27/34 | 9/10 / 8/10 |
| whisper.cpp base 5 beams / greedy | 3/6 / 2/6 | 6/6 / 5/6 | 24/34 / 20/34 | 5/10 / 5/10 |
| whisper.cpp tiny 5 beams / greedy | 1/6 / 2/6 | 5/6 / 5/6 | 19/34 / 17/34 | 5/10 / 4/10 |
| sherpa-onnx Whisper tiny | 1/6 | 4/6 | 16/34 | 4/10 |

Six drug occurrences means one word moves a result by 17 points. Many misses are one or two letters off ("amoxiciline",
"ibuprophène"), some are far ("libby profene"). Everything below was measured on the same 95 sentences with a vocabulary **taken from
those sentences**, so each gain is an **upper bound** for a vocabulary known in advance; no test used a vocabulary that does not
contain the answer, and the false-correction rate on free text is unknown [C-17 to C-20] (runs `*-t4-*`, `termstudy-vs-clean-run.md`
in each folder, D-036).

| Technique (all OFF by default) | Result on the clean run (95 sentences) | Verdict |
|---|---|---|
| Dictionary post-correction, **strict** (6+ letters, same first letter, one edit up to 11 letters) | 44 changes, **0 words broken in all 9 configurations**; Parakeet drug 4/6 to 6/6 and WER 2.3 to 2.1 %; Canary 4/6 to 5/6; whisper.cpp small 3/6 to 5/6 (3.9 to 3.5 % and 4.5 to 3.9 %); base greedy 2/6 to 4/6 | helps, cheap, engine-independent; cannot fix far misses |
| Post-correction, medium | a little more fixed (Canary 4/6 to 6/6), 0 broken, but silently replaces a correct look-alike drug ("prednisolone" to "prednisone") | hazard documented (I-045) |
| Post-correction, **loose** | 365 changes, 26 to 38 words broken per configuration, Parakeet WER 2.3 to 5.9 % ("matin" to "main" 63 times, "après" to "API REST" 27 times) | regression; study only |
| whisper.cpp initial prompt from the vocabulary | small 5 beams 3.9 to 3.3 % (11 fixed, 7 broken), drug names 3/6 to 4/6 (5 beams) and 3/6 to 3/6 (greedy): no reliable gain; base 11.4 to 9.8 % greedy; **tiny worse** (16.1 to 17.1 %, 54 words broken); inference 25 to 40 % slower; slightly worse on sentences without key terms | useful for small and base only; not for drug names |
| Prompt + strict correction (whisper.cpp small 5 beams) | 3.9 to 3.1 %, drug 3/6 to 5/6, 5 words broken (all from the prompt) | best combination on whisper.cpp |
| Parakeet hotwords, score 1.5 | drug 4/6 to 5/6, WER 2.3 to 2.7 %, 4 words broken, elision lost ("d amoxicilline"); needs a surrogate SentencePiece vocabulary (the download has none) | not recommended |
| Parakeet hotwords, score 3.0 | drug 6/6 but WER 5.1 %, 30 words broken, vocabulary words inserted into unrelated sentences, one runaway | regression |

**What this means.** If a drug-name safeguard is wanted, the evidence supports **strict post-correction with a curated vocabulary,
every replacement logged and shown to the user**, and keeping engine biasing off. The risk is stated plainly: a dictionary cannot
tell a misspelling from a rare real word, and in a medical text nobody re-reads a plausible word. The vocabulary is clinical content
to maintain. This is a recommendation of D-036, not a product decision. Canary and sherpa-onnx Whisper have no biasing
mechanism in this runtime.

---

## 5. TTS assessment

All voices run through one provider over sherpa-onnx (`speech/tts.rs`); speech is never played automatically.

### 5.1 Voices examined (nine packages installed; ratings are commercial-use ratings, section 9)

| Voice | Lang | Gender | Rating | Phonemizer | Package (MB) | Warm RTF | Peak memory (MB) | Cold load (s) |
|---|---|---|---|---|---|---|---|---|
| Piper siwis medium | fr | female (as heard; the card is silent) | review | espeak-ng | 67.2 | 0.045 to 0.050 (also 0.045 to 0.079 in the first run) | 258 | 0.80 |
| Piper gilles low (16 kHz) | fr | male (as heard) | excluded | espeak-ng | 67.1 | 0.034 to 0.037 | 212 | 0.78 |
| Coqui VITS css10 | fr | not documented | attribution | none | 67.0 | 0.046 to 0.051 | 231 | 0.97 |
| Piper mls medium (converted locally) | fr | 125 speakers | attribution | espeak-ng | 76.7 | about 0.09 (smoke test) | 298 | n/a |
| Kokoro v1.0 int8, `ff_siwis` | fr | female (documented) | review | espeak-ng | 132.3 | **1.10 to 1.28** (run 2); 1.61 to 2.06 (run 1) | 432 | 1.8 |
| Piper ljspeech medium | en | single speaker, not documented | clear | espeak-ng | 67.2 | 0.052 to 0.064 | 248 | 1.29 |
| Piper libritts_r medium | en | 904 speakers, gender not documented | attribution | espeak-ng | 82.0 | 0.056 to 0.061 | 234 | 0.96 |
| Piper kristin medium | en | single speaker | clear | espeak-ng | 67.3 | 0.055 to 0.058 | 197 (median) | not recorded |
| Kokoro v1.0 int8, `af_heart`, `am_adam` | en | female, male (documented) | review | espeak-ng | (same package) | 1.10 to 1.33 (run 2); 1.55 to 1.98 (run 1) | 453 to 456 | 1.7 |

Sources: `20261008-094413-tts-m6-licence-focus` (8 voices in one run, 3 repetitions, 4 sentences of 30 to 177 characters, preflight 23 %, a
window of the application that was not started by the test was open) for the Piper, Coqui and Kokoro figures; `20261008-085403-tts-m6`
for the first Kokoro and Piper ranges; `20261008-141002-tts-m6d-overhead` for kristin and the paragraph-mode RTF (5 repetitions, preflight
10.5 % and 21.8 %). Piper and Coqui are **13 to 29 times faster than real time** (RTF 0.034 to 0.079 over the two runs), Kokoro int8
is **slower than real time by a factor of 1.1 to 2.1 depending on the run** (a 40 to 55 % difference between runs for the same voices)
[C-21, C-22]. The spread between repetitions is 1 to 51 % in the second run (up to 89 % in the first, on generation times of 66 to
400 ms), larger than the speech-to-text spread because the times are so short.

### 5.2 Speed control (Plan: "adjustable speech rate")

The speed is passed to the library (0.5 to 2.0). Audio duration of one French sentence, one run per setting, Piper siwis / Kokoro
`ff_siwis` (first M6 smoke test, project log): speed 0.5: 6,578 / 8,664 ms; 0.8: 4,674 / 5,060; 1.0: 4,040 / 4,079; 1.25: 3,448 / 3,415;
1.5: 3,111 / 2,566; 2.0: 2,600 / 2,053 ms. Kokoro follows the setting almost linearly; Piper does not (2.0 gives 0.64 of the duration,
0.5 gives 1.63): the setting is not a true factor for Piper [C-23]. The owner found speed 1.5 acceptable on all eight voices of session 1
(scores 4 or 5).

### 5.3 The owner's listening results (opinion, one listener; full text in `docs/TTS_LISTENING_NOTES.md`)

Scale 1 (very bad) to 5 (very good), blind codes, three or four clips per voice [C-25].

| Session 1 voice | Clarity / natural / accent | Owner's note (gist) |
|---|---|---|
| Piper siwis (fr) | 5 / 5 / 5 | "very pleasant and clean, very good intonation" |
| Piper gilles (fr) | 4 / 5 / 3 | peculiar accent, rolls the R, "overall very correct" |
| Coqui css10 (fr) | 3 / 4 / 2 | strong accent, "Rendez-vous" mispronounced, digits not spoken |
| Kokoro `ff_siwis` (fr) | 4 / 3 / 4 | "unpleasant hiss or crackle" |
| Piper ljspeech (en) | 5 / 4 / 5 | "very good, good intonation" |
| Piper libritts_r (en) | 5 / 5 / 5 | "even more natural than the previous voice" |
| Kokoro `af_heart`, `am_adam` (en) | 4 / 4 / 4 | "slight hiss or crackle" (I-057) |

The owner's choice in session 1 was Piper siwis, gilles, ljspeech and libritts_r. Session 2 (the owner called the page's voices
"mediocre"): converted `mls` speakers 1 to 2 on naturalness (rejected); Coqui with the number rewriting ON 3 / 3 / 3 and "numbers well
said", with it OFF clarity 1; English ljspeech 4 / 4 / 4, libritts_r 3 / 3 / 3, kristin 3 / 2 / 3. In the owner's longer interface tests
(own texts, rewriting ON) the best French voices are siwis (female) and gilles (male), and in English libritts_r at speed 0.9, with
kristin and ljspeech "correct"; "all the other voices are not satisfactory". Drug names were heard well pronounced on 7 of 8 voices
(session 1); a recogniser round trip had counted many more failures, so it is only a rough sanity check.

### 5.4 Read-along and text normaliser (D-042, D-043)

- **Read-along**: the text is split into sentences, each is synthesised on its own and the pieces are joined into one WAV with
  measured sentence times (250 ms pause after a sentence, 600 ms after a paragraph); the sentence being read is shaded in grey (light
  and dark), with automatic scrolling and a "Follow reading" switch; nothing plays by itself. Verified: 163 Rust unit tests, 16
  front-end tests, **33 of 33 UI checks** (development and packaged app, `scripts/ui_test_readalong.mjs`, muted audio, driven over the
  DevTools protocol). Per-sentence synthesis costs a few percent (5-sentence texts -1 to +4 %, 10-sentence texts -7 to +5 %, inside the
  run-to-run spread) and the pauses make the audio 7 to 9 % longer [C-26, C-27].
- **Normaliser** (numbers, dates, times, units, abbreviations to words, French and English, no new dependency): ON by default for every
  voice since D-043 (owner's decision). On the Coqui voice, a recogniser round trip heard the key numbers 0/36 times with the normaliser
  off and 28/36 and 30/36 times with it on (two complete runs); the owner confirmed by ear that it speaks numbers but stays at 3 out of 5
  [C-28]. Known limits (I-061): acronyms not spelled, Roman numerals, ambiguous "10.30" and "5 100", Swiss number words, English rules
  never heard on a character-based English voice.
- Synthesis is not deterministic (the same text gives slightly different audio): recogniser checks need repetitions [C-24].

### 5.5 Assessment

English TTS meets the requirement with licence-clean Piper voices at interactive speed. French TTS meets the requirement of quality only
with voices whose licence lineage is unresolved; the clean French voices are judged unsatisfactory by the owner. Male and female options
exist for both languages in capability (French female and male Piper voices, English multi-speaker `libritts_r`, documented genders on
Kokoro), but the licence-clean set has no good French male voice (I-049). Pause, resume and seek come from the player; resume and
pause were checked in the automated read-along test, not with a real click. The WAV export was reported working by the owner (not
re-observed) [C-30].

---

## 6. Performance

Machine: Intel Core 7 150U (10 cores, 12 threads), 23.6 GB RAM, Windows 11 Home x64, no discrete GPU, release builds, background CPU
about 20 % (not an idle machine).

| Item | Measured value | Source |
|---|---|---|
| Speech-to-text speed, memory, cold start | section 2.2 | clean run, timing study |
| Text-to-speech speed, memory, cold start | section 5.1 | TTS runs |
| Installer | 8,858,030 bytes (8.45 MiB), SHA-256 `589EAF70...54B6C37D` (re-computed in M8: identical); installed 34,918,850 bytes in 7 files; no model, voice or phonemizer data inside | M7 log entry |
| Window visible after launch | 119 to 132 ms (6 starts; once 265 ms after a re-install); the empty native window, not the content | M7 |
| Model table rendered | median 749 ms (686 to 829, 5 starts, 22 to 38 % CPU), warm file cache | M7 |
| Idle memory of the packaged app (first build, before the CSP) | 8 processes, private memory 179.5 MB, working set 398 MB (shared pages counted several times); `speechlab.exe` alone 29.9 MB working set | M7 |
| Release build time | 115 to 155 s with the dependencies already compiled (**not** a from-scratch build) | M7 |
| Model sizes (archives) | Parakeet 487 MB, Canary 154 MB, sherpa Whisper tiny 116 MB, whisper.cpp tiny 78 MB, base q5_1 60 MB, small q5_1 190 MB, Silero VAD 0.6 MB; voices 67 to 132 MB | `models-manifest.json` |

Not measured: start-up after a reboot (cold file cache), memory during a packaged-app transcription, behaviour on battery, under heavy
load, thermal throttling over a long session, any other CPU. The RAM need of Parakeet (1.6 to 1.9 GB) on a machine with 8 GB has not
been tried [C-32, C-33, C-52].

---

## 7. Windows and macOS compatibility

**Update 2026-10-09 (D-048, I-075): the macOS work is on hold until further notice, the owner having no Mac to implement and test a
macOS version.** Every macOS cell below stays NOT VERIFIED; nothing here may be read as macOS support. SpeechLab is frozen and the
integration into AssistantCabinetAI is planned in `docs/integration/`.

| Capability | Windows 11 x64 | macOS (Apple Silicon, Intel) |
|---|---|---|
| Build from source | VERIFIED (Tauri 2.12.1, sherpa-onnx 1.13.8 static libs, whisper.cpp v1.9.4 with CMake and NMake) | NOT VERIFIED |
| Installer | VERIFIED: per-user NSIS, no administrator rights, silent install in 2.1 s, silent uninstall keeps user data (one registry key left, I-069) | NOT VERIFIED |
| Both speech-to-text engines in the packaged app | VERIFIED (word-for-word transcription of a generated clip by Parakeet and whisper.cpp base; sidecar found next to the executable) | NOT VERIFIED |
| Speech synthesis in the packaged app | VERIFIED (valid WAV; the phonemizer data inside the downloaded voice package is found) | NOT VERIFIED |
| Microphone in the release origin | VERIFIED: native prompt, capture works, permission still granted after a restart (answered through the debugging protocol, not by a hand click); closes I-024 for Windows | NOT VERIFIED (`NSMicrophoneUsageDescription`, hardened-runtime entitlement) |
| Audio import by the web view's decoders | VERIFIED for WAV, MP3, Ogg/Opus | NOT VERIFIED; Ogg/Opus in WKWebView is doubtful (I-026) |
| Content Security Policy | VERIFIED strict policy enforced (D-045) | NOT VERIFIED |
| Signing | **unsigned**; SmartScreen behaviour NOT VERIFIED (I-068) | NOT VERIFIED (Developer ID, notarisation) |
| Clean machine (no Visual C++ runtime, maybe no WebView2) | **NOT VERIFIED**: the sidecar imports MSVCP140, VCRUNTIME140, VCOMP140 (I-070) | n/a |
| Windows 10 | **NOT VERIFIED** (only Windows 11 tested) | n/a |
| UI automation | DevTools protocol script works on WebView2 | does not work on WKWebView: manual checklist |

Integration difficulties and packaging constraints found: whisper.cpp is built as shared libraries, so the sidecar needs four DLLs
(a static rebuild would give one file and remove the Visual C++ dependency, recommended, not done); every `cargo build` of the package
needs the staged sidecar (I-064); the web view's microphone permission is per origin (the development and release origins differ);
the antivirus on the development machine intercepts TLS and broke the crate's library download (I-009, worked around); the installer
creates Desktop and Start Menu shortcuts by default. macOS steps, expected file names (`whisper-cli-aarch64-apple-darwin`,
`-x86_64-apple-darwin`), the archive names the crate selects and the notarisation flow are in `docs/MACOS_VALIDATION.md`; **every line
there is NOT VERIFIED** [C-36 to C-41].

---

## 8. Privacy and offline operation

| Statement | Status | Evidence |
|---|---|---|
| Transcription and synthesis need no network | VERIFIED for the application's own code | the inference modules contain no network code (the only network code is `download.rs`, used by the explicit model installer); a firewall rule blocking all outbound traffic of the installed `speechlab.exe` and `whisper-cli.exe` was active while both speech-to-text engines and the speech synthesis worked (19 of 20 checks, the failure being the cancel banner I-065) |
| The block is effective | VERIFIED | a model download started from the app failed within 208 ms with "os error 10013" |
| No audio or transcript sent by the app's processes | VERIFIED in a window | network endpoints of the application's process tree sampled 88 times over about 40 s: no connection from `speechlab.exe` or `whisper-cli.exe`. A sampling window, not a packet capture |
| Downloads separated from inference | VERIFIED by design and tests | models are installed only by an explicit user action, with a pinned SHA-256; the installer carries no model |
| The system web view is offline | **NOT PROVEN** | the WebView2 network service kept one established HTTPS connection to a Microsoft address range (`2620:1ec:33::11:443`) while the app ran, even with the app's processes blocked; purpose NOT VERIFIED (I-066) |
| Whole-machine network off | **NOT DONE** | the per-program block was used instead, because disconnecting the machine would cut the working session. The prompt T8 asked for a "real test with the network disabled": steps below |
| Recorded audio stays local | VERIFIED by design | clips live in the app data folder, are listed and deletable in the UI, file commands refuse paths outside it (D-023) |
| No real patient data | VERIFIED | all sentences are invented, the audio is the owner's own voice |

Steps to complete the proof (NOT VERIFIED today): on the installed app with models already downloaded, switch the machine's Wi-Fi and
Ethernet off (or use a virtual machine without a network adapter), run one transcription with each engine and one synthesis, and
record the result; separately capture traffic with a packet tool during a normal session to see what the web view runtime sends, or
start the web view with background networking disabled through `additionalBrowserArgs` and repeat the sample.

---

## 9. Licensing

**Research, not legal advice.** Ratings: `clear` = public domain or no obligation; `attribution` = commercial use allowed with a
credit or licence notice; `review` = one point needs a lawyer; `excluded` = non-commercial or copyleft data or a licence the owner
has not accepted. "Source" says where the information was read; "(M0)" is the feasibility report of 2026-10-06, "(card)" a model card
read on the date in `docs/TTS_LICENSES.md`, "(manifest)" `src-tauri/models-manifest.json`, "(cargo)" the declared licence field from
`cargo metadata` in M8.

### 9.1 Software and runtimes

| Item | Licence | Commercial use | Redistribution / attribution | Source | Rating |
|---|---|---|---|---|---|
| SpeechLab application code (this repository) | **none stated: no `LICENSE` file** | owner's decision | n/a | repository listing (I-072) | open |
| Tauri 2.12.1 | Apache-2.0 OR MIT | yes | notices travel with the binary | (cargo), (M0) | attribution |
| React 19, react-dom, scheduler; `@tauri-apps/api` | MIT; Apache-2.0 OR MIT | yes | notices | `pnpm licenses list --prod` | attribution |
| 293 Rust crates in the resolved tree (Windows, normal and build) | declared expressions: MIT and/or Apache-2.0 alone (235), Unicode-3.0 (19, one combined), Unlicense or MIT (10), BSD-3-Clause (7), ISC alone or in combinations (6), Zlib alone or in combinations (6), CC0/MIT-0/Apache-2.0 (2), 0BSD (1), **MPL-2.0 (5: cssparser, cssparser-macros, dtoa-short, option-ext, selectors)**, CDLA-Permissive-2.0 (2: webpki-roots); no GPL, LGPL or AGPL declared | yes | notices; MPL-2.0 is file-level copyleft (source of modified MPL files only) | (cargo), declared fields only, texts not read | attribution (MPL: review) |
| sherpa-onnx 1.13.8 (crate and prebuilt static libraries) | Apache-2.0 | yes | NOTICE and licence | (M0), (cargo) | attribution |
| ONNX Runtime (inside the sherpa-onnx libraries) | MIT | yes | notice | (M0) | attribution |
| **espeak-ng** (phonemizer inside the standard sherpa-onnx libraries) | **GPL-3.0** | **unresolved** | **found in the packaged `speechlab.exe` (95 occurrences of "espeak", M8 byte scan); 65 in the development STT-only example; 0 in a build against the official `no-tts` libraries (M2 experiment, a workaround not supported by the crate)** | M2, M8 scan, D-012, I-051 | **review (high)** |
| whisper.cpp v1.9.4 (ggml ships inside its repository; its licence file was not read separately) | MIT | yes | notice; shipped as sidecar + 4 DLLs | (M0) | attribution |
| Silero VAD model (support model) | MIT | yes | notice (read in the upstream LICENSE, 2026-10-08) | D-035 | attribution |
| Visual C++ runtime DLLs imported by the sidecar | Microsoft redistributable, **terms not read** | to read | the installer does not carry them (I-070) | import tables | review |
| WebView2 runtime and its bootstrapper (downloaded by the installer's default) | Microsoft, **terms not read** | to read | system component | M7 | review |
| NSIS (installer builder), MSVC Build Tools | **not read**; Build Tools are free but not open source (D-003) | to read | build-time tools | D-003 | review |

### 9.2 Speech-to-text models (none bundled; downloaded on user action)

| Model | Licence | Commercial use | Redistribution / attribution | Source | Rating |
|---|---|---|---|---|---|
| Parakeet TDT 0.6B v3 (int8 ONNX conversion by the sherpa-onnx project) | CC BY 4.0 | yes | credit the licensor, give the licence link, **indicate changes** (the ONNX/int8 files are a third-party conversion); training-data provenance **not read** | (manifest), (M0) | attribution |
| Canary 180M flash (int8 ONNX conversion) | CC BY 4.0 | yes | same as above | (manifest), (M0) | attribution |
| Whisper weights (tiny, base, small, large-v3-turbo; ggml conversion by ggml-org, ONNX by sherpa-onnx) | MIT per the manifest; one model card says Apache-2.0 (never reconciled, both permissive) | yes | notice | (manifest), (M0) | attribution |
| Not retained: French streaming zipformer and Kroko models | licence **unknown** (card returned HTTP 401, I-004) | unknown | treat as unusable until read | (M0) | review |

### 9.3 Text-to-speech voices and their data

| Voice | Data licence | Lineage (from the card) | Phonemizer | Rating |
|---|---|---|---|---|
| Piper en_US **ljspeech** medium | public domain (LJ Speech) | trained from scratch | espeak-ng | **clear** |
| Piper en_US **kristin** medium | public domain (LibriVox) | trained from scratch | espeak-ng | **clear** |
| Piper en_US **libritts_r** medium | CC BY 4.0 (LibriTTS-R) | no fine-tuning line | espeak-ng | attribution |
| **Coqui VITS fr css10** | LibriVox public-domain recordings (CSS10); model BSD-3-Clause | not stated by the card | **none** (character-based) | attribution |
| Piper fr_FR **mls** medium (converted locally, `packaging: local`) | CC BY 4.0 (MLS, OpenSLR 94) | from scratch | espeak-ng | attribution (judged unusable by ear) |
| Piper fr_FR **siwis** medium | CC BY 4.0 (SIWIS; "usable for any purpose") | **fine-tuned from the English Lessac voice** (Blizzard 2013 research licence) | espeak-ng | **review** |
| Piper fr_FR **gilles** low | CC0 (card) | **fine-tuned from the English Ryan voice** (CC BY-NC-SA 4.0) | espeak-ng | **excluded** |
| Kokoro v1.0 int8 (Apache-2.0 weights) | audio under Apache/MIT, **synthetic audio from closed TTS services (terms unread)**, SIWIS CC BY 4.0, Koniwa CC BY 3.0 | n/a | espeak-ng | **review** |
| Piper fr_FR upmc; tom; tjiho; miro; Piper en_US lessac, amy, alan, ryan, hfc_female; MMS fra | CC BY-SA from Lessac; AGPL-3.0; AGPL-3.0; non-commercial (synthetic data); research licence / unread / CC BY-NC-SA; CC BY-NC | various | espeak-ng or n/a | excluded or review, never installed |
| Chatterbox, Chatterbox-French, Qwen3-TTS | MIT / CC BY 4.0 / Apache-2.0 weights; **training-data licences unstated or non-commercial (Emilia)**; voice cloning; Chatterbox watermarks every output; slower than real time on a laptop CPU as reported (not measured) | n/a | not sherpa-onnx | review, not retained |
| Kitten, Supertonic, Pocket TTS | partly unreadable cards | n/a | n/a | not rated |

Piper `_base_model` checkpoint (CC BY 4.0, trained from scratch on LibriTTS-R, MIT repository) is the clean parent for re-training
siwis and gilles (D-043); training code `piper1-gpl` is GPL-3.0 and is a training tool, not shipped. Sources: voice cards read
2026-10-08, the Blizzard 2013 licence page, the Piper maintainer's statements (discussions 271 and 94), the SIWIS README, the
sherpa-onnx release asset list (`docs/TTS_LICENSES.md`).

### 9.4 Datasets and test material

| Item | Licence / status | Note |
|---|---|---|
| 95 reading sentences (`benchmark/scripts/`) | written from scratch for this project; invented, not medical or legal advice | no external text |
| Owner's recordings (`benchmark/audio/`, git-ignored) | own voice, owner's consent; `committable: false` | only metadata and references are committed |
| Private clips of third parties | none imported (accent evaluation dropped, D-046) | `bench import` stays in the code, unused |
| Voices of the TTS corpora | see 9.3 | not redistributed by this project |

### 9.5 Flags the owner should keep in view

1. **espeak-ng (GPL-3.0) is inside every build that synthesises speech**, including the current installer (I-003, I-010, I-051, I-054,
   D-012). Ways out, none tried in a product: an STT-only build with the `no-tts` libraries (identical STT output in the M2 experiment);
   speech synthesis in a separate executable with its own GPL notice; a legal clearance; a character-based voice on a runtime without
   the phonemizer (needs a build of sherpa-onnx without it, not tried).
2. **NeMo models (Parakeet, Canary) are CC BY 4.0**: credit, licence link and a statement of changes are required; the application has
   no credits view and the installer no licence page yet (I-067).
3. **Piper siwis (`review`) and gilles (`excluded`) have restricted parent checkpoints**; the owner keeps them for development and will
   re-train them from the clean base before any commercial release (D-043, I-063: a **commercial-release blocker**). Whether a free beta
   handed to others counts as non-commercial use is a legal question.
4. **The `excluded` and `review` voices must not be a default of any commercial build** (D-039, D-041 as amended by D-043): Kokoro,
   siwis, gilles, mls (rejected by ear), and everything listed as not retained.
5. **Models are never bundled** (installer carries none): the application downloads them from third-party hosts on user action, which
   moves, but does not remove, the attribution duty (the app shows each model's licence and each voice's rating).
6. The application itself has **no licence file** (I-072).

### 9.6 What a lawyer must read

- GPL-3.0 as applied to a binary that statically links espeak-ng, with and without a character-based voice that never calls it; the
  separate-executable option; whether the installer needs the GPL text and a source offer.
- Whether weights fine-tuned from the Lessac (Blizzard 2013 research licence) and Ryan (CC BY-NC-SA 4.0) checkpoints inherit those
  terms, and whether re-training from the CC BY 4.0 base is sufficient evidence of a clean lineage.
- The Blizzard licence's clauses on derived models (only part of the page was read).
- CC BY 4.0 notice wording for the NVIDIA models converted to ONNX/int8 by a third party, and the licences of their training data (not read).
- The terms of the closed TTS services whose output trained Kokoro.
- Redistribution terms of the Visual C++ runtime, the WebView2 bootstrapper and the installer builder; the five MPL-2.0 crates.
- Whether handing a free beta to third parties is "commercial use" for the restricted voices.
- The licence of the SpeechLab code itself and of any code later moved into AssistantCabinetAI.

---

## 10. Architecture recommendation

### 10.1 The options of `Plan.md` section 12 against the evidence

| Option | For | Against | Verdict |
|---|---|---|---|
| whisper.cpp as primary STT | MIT; real cancellation; 0.2 to 0.5 GB; no long-audio failure | `small` (the only accurate size) runs at RTF 1.1 to 1.3 on this CPU: not interactive; `base` is 8 % WER; sidecar with 4 DLLs and a Visual C++ dependency; turbo and accelerated builds untested | not supported as the default on this hardware |
| sherpa-onnx as primary STT | Parakeet: best WER, RTF 0.10, no failure; one in-process library; runs fully offline | 1.6 to 1.9 GB, 7 to 10 busy cores, no cancellation, CC BY 4.0 attribution, the full libraries carry GPL-3.0 espeak-ng, Canary and Whisper-tiny failure modes | supported as the default, with the open points below |
| **Both engines as interchangeable providers** | the provider contract already exists and both engines run end to end in the Tauri app; the failure profiles differ (cancellation, memory, licence); no coupling | two integration paths to maintain, a sidecar and a runtime dependency | **recommended** |
| Different engines for different scenarios | would match "queries versus dictation" | the evidence shows no scenario where whisper.cpp small beats Parakeet on accuracy or speed on this machine; the only differences are cancellation, memory and licence | **not supported by the evidence** |
| Alternative if neither meets requirements | needed for **French TTS** | for STT, one configuration meets the requirement on this speaker | needed for French voices only |

### 10.2 Recommendation (provisional)

1. **Architecture**: keep the engine-agnostic contract of `speech/types.rs` and the Tauri command layer as the integration boundary
   (the future application consumes the traits and the command contract; engines and models stay behind adapters).
2. **Speech-to-text default**: sherpa-onnx Parakeet TDT v3 int8, greedy. Add before use: an output guard (reject outputs far from the
   audio duration, I-035), no chunking for clips up to 25 s (provably unchanged) and, if dictation beyond 30 s is a requirement,
   re-test chunking on Parakeet with more clips (neutral on 3 dictations; the 25 s limit is wrong for Canary).
3. **Second provider**: keep whisper.cpp small (5 beams) behind the same contract, with the optional initial prompt and strict
   post-correction, as the engine with cancellation, a small footprint and an MIT licence. Rebuild it statically before any
   distribution. Drop Canary, sherpa-onnx Whisper tiny and whisper.cpp tiny/base from any product shortlist.
4. **Drug-name safeguard**: if wanted, strict post-correction with a curated, logged and visible vocabulary; never the loose preset;
   no engine biasing by default.
5. **Text-to-speech**: sherpa-onnx with Piper `ljspeech`/`kristin` (`clear`) and `libritts_r` (`attribution`) for English, text
   normaliser ON, read-along kept. **French: no licence-clean voice is both clean and good.** Interim for a beta: Coqui css10 (3 out of 5,
   rated `attribution`) or English only; target: siwis and gilles re-trained from the clean base and re-listened blind (D-043).
6. **No build with speech synthesis ships before the espeak-ng decision** (section 9.5, point 1).

### 10.3 Why this is not a proven winner, and what would settle it

| Open question | Why it is open | What would settle it |
|---|---|---|
| Is Parakeet better than whisper.cpp small for real users? | +1.6 points (+0.1 to +3.1) on one speaker | a panel of speakers (voices, microphones, noise, conversational speech) through the same `bench run`; decide a margin in advance |
| Is Parakeet's footprint acceptable? | 1.6 to 1.9 GB and 7 to 10 busy cores measured on a 23.6 GB machine only | the same benchmark on the weakest target machine (8 GB RAM, older CPU) and on battery |
| Does whisper.cpp become competitive with acceleration or turbo? | `large-v3-turbo` never run; no Vulkan, Metal or Core ML build tested | install turbo, build with an accelerator, run `bench run --models ...` on the same sentences and, on a Mac, the Metal build |
| Which engine for streaming dictation? | only whole-clip recognition was evaluated | a streaming model evaluation (the French streaming zipformer licences are unknown, I-004) |
| French TTS quality with a clean licence | re-training not tried | re-train from `_base_model` on a GPU outside this machine and listen blind |
| GPL phonemizer | legal question, no technical alternative tried | lawyer's reading; test a split into two executables or an STT-only build |
| macOS | no Mac | `docs/MACOS_VALIDATION.md` |
| Accents | dropped by the owner | out of scope; if the owner changes their mind: public clips with a licence through `bench import` |

---

## 11. Integration roadmap for AssistantCabinetAI

Each step lists what must be true first. Nothing here touches the main repository from this project.

**Update 2026-10-09: this section is superseded in detail by `docs/integration/03-integration-plan.md`** (phases SP-0 to SP-8,
decisions Q-01 to Q-09, analysis of both projects, review of the owner's draft). SpeechLab is frozen (D-048); macOS is on hold.
The table below is kept as the roadmap as it stood at the end of M8.

| # | Step | Precondition |
|---|---|---|
| 1 | Settle the licence position: decide the GPL phonemizer strategy, choose the licence of the SpeechLab code, decide how credits are shown | a lawyer's reading, or the owner's written decision (section 9.6); budget if needed (D-041) |
| 2 | Small fixes in the lab: cancel banner (I-065), credits and licence view from the manifest and an installer licence page (I-067), output guard (I-035), static whisper.cpp rebuild (I-070) | owner's approval of each change (proposed in M7, not chosen) |
| 3 | Integrate the **speech-to-text contract** into AssistantCabinetAI (traits, command contract, model manager with pinned checksums) with Parakeet as default and whisper.cpp as second provider; models downloaded, never bundled | step 1 for the NVIDIA attribution; a decision on the interface for transcripts (never auto-sent to an AI agent, never auto-played) |
| 4 | Widen the evidence: speaker panel, noise, conversational speech, a weak target machine, optional turbo and accelerated builds | volunteers with consent; a second machine; owner's time |
| 5 | **English TTS** with `clear`/`attribution` voices, normaliser and read-along | step 1 (espeak-ng) decided; credits (step 2) |
| 6 | **French TTS**: re-train siwis and gilles from `_base_model`, blind listening, update ratings; or a licensed on-device voice SDK | a GPU outside this machine (8 GB video memory is the reported minimum), the owner's decision on cost (`docs/TTS_LICENSES.md` has prices read, not confirmed) |
| 7 | **Windows distribution**: code-signing certificate, SmartScreen check, clean-machine test (no Visual C++ runtime, no WebView2), interactive uninstaller check, Windows 10 test | certificate purchase; a clean virtual machine or Windows Sandbox |
| 8 | **macOS (ON HOLD since 2026-10-09, no Mac available, D-048)**: run `docs/MACOS_VALIDATION.md` on Apple Silicon and Intel, Metal build of whisper.cpp, Info.plist and entitlements, Developer ID signing and notarisation, Ogg/Opus import alternative | a Mac; an Apple developer membership (price not checked) |
| 9 | Optional product features measured before use: drug-name post-correction on free text (false-correction rate), chunking per engine, streaming | more speakers and real texts; owner's decision on clinical content maintenance |

---

## 12. Appendix A: claims table

VERIFIED = observed in this repository or on this machine, with the evidence named. NOT VERIFIED = not observed; the steps are
given. "Log" = `docs/PROJECT_LOG.md` entry.

### A.1 Speech-to-text

| ID | Claim | Status | Evidence / steps |
|---|---|---|---|
| C-01 | Parakeet WER 2.3 % (1.4 to 3.4) on 95 sentences of one speaker; others in section 2.2 | VERIFIED | `20261007-215116-full-owner-reps1-clean/summary.md`; `python scripts/bootstrap_ci.py <folder>` |
| C-02 | Parakeet vs whisper.cpp small 5 beams +1.6 (+0.1 to +3.1); vs Canary not distinguishable | VERIFIED | `bootstrap_ci.py` output, re-run in M8 |
| C-03 | Accuracy is deterministic and load-independent | VERIFIED | 855/855 identical (clean vs disturbed), 360/360 (clean vs timing), 828/828 (short-vad), 27/27 (long-whole); `scripts/compare_runs.py`, `chunking_study.py same` |
| C-04 | Speed: Parakeet RTF 0.099 to 0.114, small 1.12 to 1.32 | VERIFIED | clean run and `20261008-035938-timing-3reps`; margin 10 to 20 % |
| C-05 | Spread: median CV 0.7 to 3.5 % within a run; up to 20 % (and 1 to 48 % in one comparison) between runs; cause unknown | VERIFIED (effect), NOT VERIFIED (cause) | `scripts/timing_study.py`; I-012, I-042. Cause: log the CPU frequency and load continuously during two runs |
| C-06 | sherpa-onnx 7.4 to 10.4 busy cores, whisper.cpp 3.5 to 3.9 | VERIFIED (effect), NOT VERIFIED (cause) | I-036. Cause: set the ONNX Runtime intra/inter-op threads in the session options and re-measure |
| C-07 | Peak memory: Parakeet 1.6 to 1.9 GB, small 0.4 to 0.5 GB | VERIFIED (sampled lower bound) | summary tables |
| C-08 | Canary: garbage output (WER 517 % on one sample), dropped endings, degradation on 24 s pieces | VERIFIED | I-019, I-035, I-041, runs `*-long-*` |
| C-09 | sherpa Whisper tiny: 30 s hard limit, repetition loops | VERIFIED | library message; `long-whole`/`long-vad`; I-018 |
| C-10 | Parakeet and whisper.cpp: no long-audio failure on 3 dictations and 2 private clips | VERIFIED, low power | runs `*-long-*`; more clips and speakers needed (NOT VERIFIED beyond) |
| C-11 | French/English WER with intervals (section 3) | VERIFIED | `python -I -X utf8 scripts/wer_by_language.py <clean run>` |
| C-12 | Critical errors exist in every configuration; drug names found 1 to 4 of 6 | VERIFIED | summary tables, section 4 |
| C-13 | Detector precision about 80 to 85 % on a hand review of 23 flags; recall unknown | VERIFIED (small), recall NOT VERIFIED | I-034. Review all flags of one run by hand |
| C-14 | No accent figure exists | VERIFIED (absence) | D-046 |
| C-15 | Speaking speeds, conversational speech, noise, other microphones not covered | NOT MEASURED | record a panel; steps in section 10.3 |
| C-16 | Results for other speakers | NOT VERIFIED | `bench import` or the in-app recorder for each speaker, then `bench run` |
| C-17 | Strict post-correction: 0 broken words, fixes 2 more drug names (Parakeet, small) | VERIFIED (upper bound) | `20261008-065342-t4-postcorrect-strict-on-clean/termstudy-vs-clean-run.md` |
| C-18 | Prompt helps small and base, hurts tiny, slows 25 to 40 % | VERIFIED | `20261008-070031-t4-whisper-prompt` |
| C-19 | Hotwords: score 1.5 marginal, score 3.0 regression | VERIFIED | `20261008-065715-*`, `-065852-*` |
| C-20 | Loose correction is a regression | VERIFIED | `20261008-065346-t4-postcorrect-loose-on-clean` |

### A.2 Text-to-speech

| ID | Claim | Status | Evidence / steps |
|---|---|---|---|
| C-21 | Piper and Coqui RTF 0.034 to 0.079, 197 to 258 MB | VERIFIED | `20261008-085403-tts-m6`, `-094413-tts-m6-licence-focus`, `-141002-tts-m6d-overhead` |
| C-22 | Kokoro int8 slower than real time (1.10 to 2.06) | VERIFIED | same runs |
| C-23 | Speed control works on both families, differently | VERIFIED (one run per speed) | log "M6" |
| C-24 | Synthesis is not deterministic | VERIFIED | I-056 |
| C-25 | Owner's listening scores | VERIFIED as recorded; opinion of one listener | `docs/TTS_LISTENING_NOTES.md` |
| C-26 | Read-along: 163 Rust tests, 16 front-end tests, 33/33 UI checks | VERIFIED | log "M6d"; `20261008-141002-tts-m6d-overhead/ui-test-results.json`; re-run on the packaged app in M7 (33/33) |
| C-27 | Per-sentence synthesis overhead a few percent | VERIFIED | `20261008-141002-tts-m6d-overhead/overhead.md` |
| C-28 | Normaliser makes the Coqui voice speak numbers (28/36 and 30/36 vs 0/36 by a recogniser) | VERIFIED (proxy); owner confirms by ear | `normaliser-roundtrip-parakeet.txt` |
| C-29 | A good, licence-clean French voice exists | NOT VERIFIED (none found) | re-train (section 11, step 6) |
| C-30 | Pause/resume and WAV export with real clicks | NOT VERIFIED by me; WAV export reported working by the owner | do the nine UI checks of the log "M6c" by hand |

### A.3 Performance, compatibility, privacy

| ID | Claim | Status | Evidence / steps |
|---|---|---|---|
| C-31 | Installer 8,858,030 bytes, SHA-256 as stated, 7 installed files | VERIFIED | M7; SHA re-computed in M8 |
| C-32 | Window 119 to 132 ms, table 749 ms median | VERIFIED (warm cache) | M7. After a reboot: NOT VERIFIED, repeat the 6 starts |
| C-33 | Idle private memory 179.5 MB | VERIFIED (first build) | M7. CSP build not re-measured |
| C-34 | Windows 11 packaged app: both engines, TTS, cancel, first start without models, 33/33 and 23/24 checks | VERIFIED | M7 log; `scripts/ui_test_packaged.mjs`, `ui_test_readalong.mjs` |
| C-35 | Microphone permission in the release origin persists after restart | VERIFIED (click made through the debugging protocol) | M7; repeat once by hand |
| C-36 | Windows 10 | NOT VERIFIED | install on a Windows 10 machine and run the packaged-app test |
| C-37 | Clean machine without the Visual C++ runtime | NOT VERIFIED | install in Windows Sandbox or a clean VM, run a whisper.cpp transcription |
| C-38 | SmartScreen behaviour, signing requirements and prices | NOT VERIFIED | download the installer in a browser on a clean machine; read the certificate vendors' pages |
| C-39 | macOS build, run, microphone, signing | NOT VERIFIED, **on hold until further notice (no Mac, D-048)** | `docs/MACOS_VALIDATION.md` |
| C-40 | Model download from the packaged app over TLS | NOT VERIFIED (only the blocked failure observed) | install a model from the installed app on a network without interception |
| C-41 | Interactive uninstaller and its effect on the models folder | NOT VERIFIED | run it with the real data folders moved away |
| C-42 | Whole-machine network-off test | NOT DONE | section 8 steps |
| C-43 | Web view runtime connection to a Microsoft range | VERIFIED (observation), NOT VERIFIED (purpose) | I-066 |
| C-44 | The app's processes made no outside connection in a 40 s window | VERIFIED (window only) | M7, 88 samples |
| C-45 | Network code exists only in the model downloader | VERIFIED by reading and design | `download.rs`; D-013 to D-015 |

### A.4 Licensing and recommendation

| ID | Claim | Status | Evidence / steps |
|---|---|---|---|
| C-46 | The packaged executable contains the GPL-3.0 phonemizer code | VERIFIED | M8 byte scan of `src-tauri/target/release/speechlab.exe`: 95 occurrences of "espeak" (31,447,040 bytes). Steps: `python -I -X utf8 -c "print(open('src-tauri/target/release/speechlab.exe','rb').read().lower().count(b'espeak'))"` |
| C-47 | Voice ratings | VERIFIED as a reading of cards on 2026-10-08; legal effect NOT VERIFIED | `docs/TTS_LICENSES.md` |
| C-48 | Declared licences of 293 Rust crates and the front-end packages | VERIFIED (declared fields only) | `cargo metadata --format-version 1 --offline --filter-platform x86_64-pc-windows-msvc` from `src-tauri`; `pnpm licenses list --prod` |
| C-49 | NeMo models CC BY 4.0; data provenance | licence VERIFIED from the manifest and M0 cards; data provenance NOT VERIFIED | read the model cards' data sections |
| C-50 | Legal questions of section 9.6 | NOT VERIFIED | a lawyer's reading |
| C-51 | Recommendation of section 10.2 | follows from the rows above; provisional | section 10.3 lists what would settle it |
| C-52 | whisper.cpp large-v3-turbo, accelerated builds, a second machine | NOT RUN | install `whisper-cpp-large-v3-turbo-q5_0` (574 MB), run `bench run --models whisper-cpp-large-v3-turbo-q5_0 --label turbo` after `bench preflight` |
| C-53 | Streaming recognition and live latency | NOT EVALUATED | needs streaming models and a streaming provider |

## Appendix B: reproduction

```bash
cd src-tauri && cargo build --release --example bench
./target/release/examples/bench.exe check
./target/release/examples/bench.exe preflight        # exit 0 = quiet enough (limit 30 %, D-034)
./target/release/examples/bench.exe run --reps 1 --label full-owner-reps1-clean
python ../scripts/bootstrap_ci.py ../benchmark/results/<run> [--exclude en-it-05-owner]
python -I -X utf8 ../scripts/wer_by_language.py ../benchmark/results/<run>
python ../scripts/timing_study.py ../benchmark/results/<run with --reps 3>
```

The dataset audio is not in the repository (voices, git-ignored): a reader reproduces the pipeline with their own recordings of the 95
scripts through the in-app recorder, or re-scores the stored transcripts with `bench rescore` and `bench summarize` (offline).

## Appendix C: corrections made while writing this report

Cross-checking every number against its source found the following; each is also in `docs/PROJECT_LOG.md` (M8 entry).

1. The M5c entry said "1 100 reference words"; the runs contain **1,054** scored reference words (1,053 whitespace tokens), 803
   French and 251 English. The entry now carries the correction.
2. `README.md` and `docs/HANDOFF.md` said Piper and Coqui are "16 to 29 times faster than real time"; the two TTS runs give 13 to 29
   (first run) and 16 to 28 (second run). Corrected to "13 to 29".
3. `docs/HANDOFF.md` said whisper.cpp small has RTF "1.1 to 1.2"; the clean run gives 1.12 and 1.24 and the timing study 1.26 and 1.32.
   Corrected to "1.1 to 1.3".
4. `README.md` contained two control characters (a bell and a carriage return) in the sentence about the recordings folder, where two
   backslash sequences of the Windows path had been turned into control codes; repaired (I-071).
5. D-042 says the normaliser flag is true only for the Coqui voice; D-043 made it true for all voices. A note was added to D-042.
6. The packaged executable was never scanned for the GPL phonemizer (only the development example was); it was in M8 (C-46).
