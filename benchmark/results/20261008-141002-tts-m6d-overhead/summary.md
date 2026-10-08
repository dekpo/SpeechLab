# Run 20261008-141002-tts-m6d-overhead

Tool: `tts measure --compare 1` (release build), 5 repetitions, per-sentence and whole-text modes interleaved in each repetition, cold runs excluded, quiet machine (preflight 10.5 % before, 21.8 % after, never forced). Details in `config.json`; the analysis is in the M6d entry of `docs/PROJECT_LOG.md`.

| File | Content |
|---|---|
| `overhead.md` | per voice and text: generation time and real-time factor of the two modes, overhead in percent, audio length |
| `measure-*/tts-measure.jsonl`, `.md` | raw timings per repetition |
| `normaliser-roundtrip-parakeet.txt` | Coqui voice, normaliser off and on, six French sentences, three repetitions, recogniser check (28/36 and 30/36 key numbers heard on against 0/36 off over two complete runs) |
| `mls-speaker-scan-parakeet.csv` | the 125 speakers of the converted Piper `mls` voice, word error rate of the recogniser on two fixed sentences (clarity proxy, says nothing about naturalness) |
| `ui-test-results.json` | the 33 checks of `scripts/ui_test_readalong.mjs` (all passed) |
| `preflight-before.json`, `preflight-after.json` | quiet-machine checks |

Caveats: one machine, four voices measured for speed (kristin, ljspeech, libritts_r, Coqui css10 with the normaliser on); speed figures vary by 1 to 4 % within a run and more between runs (I-042); the recogniser checks inherit the recogniser's own mistakes.
