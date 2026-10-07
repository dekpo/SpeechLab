import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { MicRecorder } from "../audio/recorder";
import { samplesToWav16k, signalStats } from "../audio/wav";
import { createSample, deleteSample, listSamples, listScripts, readSampleAudio, saveClip } from "../speech/api";
import type { DatasetStatus, Sample, ScriptItem, Speaker } from "../speech/types";

const STORAGE_KEY = "speechlab.recorder.speaker";

interface FormState {
  speakerId: string;
  profile: string;
  gender: string;
  accent: string;
  /** "own" = my own voice; "other" = someone else's voice (kept private, never committed). */
  voice: "own" | "other";
}

const DEFAULT_FORM: FormState = {
  speakerId: "owner",
  profile: "native French, no marked accent",
  gender: "",
  accent: "",
  voice: "own",
};

function loadForm(): FormState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? { ...DEFAULT_FORM, ...(JSON.parse(raw) as Partial<FormState>) } : DEFAULT_FORM;
  } catch {
    return DEFAULT_FORM;
  }
}

interface Pending {
  wav: Uint8Array;
  url: string;
  durationMs: number;
  notice: string;
  warn: boolean;
}

const sanitize = (s: string) =>
  s.trim().toLowerCase().replace(/[^a-z0-9_]+/g, "-").replace(/^-+|-+$/g, "");

interface Props {
  onError: (message: string | null) => void;
}

export default function DatasetRecorder({ onError }: Props) {
  const recorder = useRef(new MicRecorder());
  const [scripts, setScripts] = useState<ScriptItem[]>([]);
  const [status, setStatus] = useState<DatasetStatus | null>(null);
  const [form, setForm] = useState<FormState>(loadForm);
  const [category, setCategory] = useState("");
  const [index, setIndex] = useState(0);
  const [recording, setRecording] = useState(false);
  const [busy, setBusy] = useState(false);
  const [level, setLevel] = useState(0);
  const [pending, setPending] = useState<Pending | null>(null);
  const [playUrl, setPlayUrl] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatus(await listSamples());
    } catch (e) {
      onError(String(e));
    }
  }, [onError]);

  useEffect(() => {
    listScripts().then(setScripts).catch((e) => onError(String(e)));
    void refresh();
  }, [refresh, onError]);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(form));
    } catch {
      /* storage may be unavailable; the form simply is not remembered */
    }
  }, [form]);

  const speakerId = sanitize(form.speakerId);
  // Scripts arrive already ordered by priority; categories keep that order.
  const categories = useMemo(() => [...new Set(scripts.map((s) => s.category))], [scripts]);

  // Start on the highest-priority category instead of "All categories".
  useEffect(() => {
    if (!category && categories.length > 0) setCategory(categories[0]);
  }, [categories, category]);
  const items = useMemo(() => scripts.filter((s) => !category || s.category === category), [scripts, category]);
  const mine = useMemo(
    () => new Map((status?.samples ?? []).filter((s) => s.speaker.id === speakerId).map((s) => [`${s.id}`, s])),
    [status, speakerId],
  );
  const sampleFor = (item: ScriptItem): Sample | undefined => mine.get(`${item.id}-${speakerId}`);
  const current = items[Math.min(index, Math.max(0, items.length - 1))];
  const recordedCount = (cat: string) => scripts.filter((s) => s.category === cat && sampleFor(s)).length;
  const totalCount = (cat: string) => scripts.filter((s) => s.category === cat).length;

  function go(delta: number) {
    setPending(null);
    setPlayUrl(null);
    setIndex((i) => Math.max(0, Math.min(items.length - 1, i + delta)));
  }

  function nextToRecord(from: number) {
    for (let k = 1; k <= items.length; k++) {
      const i = (from + k) % items.length;
      if (!sampleFor(items[i])) return i;
    }
    return Math.min(from + 1, items.length - 1);
  }

  async function start() {
    onError(null);
    setPending(null);
    try {
      await recorder.current.start(undefined, setLevel);
      setRecording(true);
    } catch (e) {
      onError(`Cannot start recording: ${String(e)}`);
    }
  }

  async function stop() {
    setRecording(false);
    setLevel(0);
    setBusy(true);
    try {
      const rec = await recorder.current.stop();
      if (rec.samples.length === 0) throw new Error("No audio was captured.");
      const stats = signalStats(rec.samples);
      const peak = Math.round(stats.peak * 100);
      let notice = `Level OK (peak ${peak} %).`;
      let warn = false;
      if (stats.clippedRatio > 0.001) {
        notice = `Clipping (peak ${peak} %, ${(stats.clippedRatio * 100).toFixed(1)} % of samples saturated): lower the microphone volume and record again.`;
        warn = true;
      } else if (stats.peak < 0.05) {
        notice = `Very quiet (peak ${peak} %): raise the microphone volume or move closer.`;
        warn = true;
      }
      const { wav, durationMs } = await samplesToWav16k(rec.samples, rec.sampleRate);
      setPending({ wav, durationMs, notice, warn, url: URL.createObjectURL(new Blob([wav as BlobPart], { type: "audio/wav" })) });
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function save() {
    if (!pending || !current) return;
    setBusy(true);
    try {
      const own = form.voice === "own";
      const speaker: Speaker = {
        id: speakerId,
        profile: form.profile.trim() || "not described",
        gender: form.gender.trim() || null,
        accent: form.accent.trim() || null,
      };
      const clip = await saveClip(pending.wav);
      await createSample({
        scriptId: current.id,
        speaker,
        source: {
          kind: own ? "own-recording" : "third-party-private",
          url: null,
          license: own
            ? "own recording, speaker is the project owner and consents"
            : "voice of a third party, private, local use only, no redistribution",
        },
        private: !own,
        clipPath: clip.path,
      });
      URL.revokeObjectURL(pending.url);
      setPending(null);
      await refresh();
      setIndex(nextToRecord(index));
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function listen(sample: Sample) {
    try {
      const bytes = await readSampleAudio(sample.id);
      setPlayUrl(URL.createObjectURL(new Blob([bytes], { type: "audio/wav" })));
    } catch (e) {
      onError(String(e));
    }
  }

  async function remove(sample: Sample) {
    try {
      await deleteSample(sample.id);
      setPlayUrl(null);
      await refresh();
    } catch (e) {
      onError(String(e));
    }
  }

  const existing = current ? sampleFor(current) : undefined;
  const speakerOk = speakerId.length > 0;

  return (
    <section>
      <strong>Dataset recorder</strong>
      <p className="hint">
        Read each sentence aloud exactly as written. The text is the reference transcript. Audio and
        samples are saved in <code>{status?.root ?? "benchmark"}</code>; audio and private samples are
        never committed to git.
      </p>

      <label htmlFor="spk">Speaker id (letters, digits, - and _)</label>
      <input id="spk" value={form.speakerId} onChange={(e) => setForm({ ...form, speakerId: e.target.value })} />
      <label htmlFor="prof">Speaker profile (honest description of this ONE person)</label>
      <input id="prof" value={form.profile} onChange={(e) => setForm({ ...form, profile: e.target.value })} />
      <label htmlFor="acc">Accent, if known (optional)</label>
      <input id="acc" value={form.accent} onChange={(e) => setForm({ ...form, accent: e.target.value })} placeholder="e.g. Suisse romande" />
      <label htmlFor="who">Whose voice is this?</label>
      <select id="who" value={form.voice} onChange={(e) => setForm({ ...form, voice: e.target.value as FormState["voice"] })}>
        <option value="own">My own voice (I consent)</option>
        <option value="other">Someone else's voice (kept private, never committed)</option>
      </select>
      {form.voice === "other" && (
        <p className="warn">Only record another person's voice with their consent. It stays in git-ignored folders and is never quoted in documents.</p>
      )}

      <label htmlFor="cat">Category</label>
      <select id="cat" value={category} onChange={(e) => { setCategory(e.target.value); setIndex(0); setPending(null); }}>
        <option value="">All categories</option>
        {categories.map((c) => (
          <option key={c} value={c}>
            {c} ({recordedCount(c)}/{totalCount(c)} recorded)
          </option>
        ))}
      </select>

      {current ? (
        <div className="sentence">
          <small>
            {index + 1 > items.length ? items.length : index + 1} / {items.length} · {current.id} · {current.language} · {current.domain} · {current.utteranceType}
            {existing ? " · ✓ already recorded (saving replaces it)" : ""}
          </small>
          <p className="big">{current.text}</p>
          {(current.keyTerms?.length ?? 0) > 0 && (
            <small>Key terms: {current.keyTerms!.map((k) => k.text).join(", ")}</small>
          )}
        </div>
      ) : (
        <p>No script found in <code>benchmark/scripts</code>.</p>
      )}

      <div>
        <button onClick={() => go(-1)} disabled={recording || busy || index === 0}>◀ Previous</button>
        <button onClick={() => go(1)} disabled={recording || busy || index >= items.length - 1}>Next ▶</button>
        {!recording ? (
          <button onClick={() => void start()} disabled={busy || !current || !speakerOk}>● Record</button>
        ) : (
          <button onClick={() => void stop()}>■ Stop</button>
        )}
        {recording && (
          <span className="meter" aria-label="input level">
            <span style={{ width: `${Math.min(100, Math.round(level * 400))}%` }} />
          </span>
        )}
      </div>
      {!speakerOk && <p className="warn">Enter a speaker id first.</p>}
      {busy && <p className="hint">Working…</p>}

      {pending && (
        <div className="resultblock">
          <p className={pending.warn ? "warn" : "hint"}>{pending.notice} Length {(pending.durationMs / 1000).toFixed(1)} s.</p>
          <audio controls src={pending.url} />
          <div>
            <button onClick={() => void save()} disabled={busy}>Save and continue</button>
            <button onClick={() => { URL.revokeObjectURL(pending.url); setPending(null); }} disabled={busy}>Redo</button>
          </div>
        </div>
      )}

      {existing && !pending && (
        <div className="resultblock">
          <button onClick={() => void listen(existing)}>Listen to the saved recording</button>
          <button onClick={() => void remove(existing)}>Delete it</button>
          {playUrl && <audio controls autoPlay src={playUrl} />}
        </div>
      )}

      {status && status.issues.length > 0 && (
        <details>
          <summary>{status.issues.length} dataset issue(s)</summary>
          <ul>
            {status.issues.slice(0, 20).map((i, k) => (
              <li key={k}>{i.sampleId}: {i.message}</li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
