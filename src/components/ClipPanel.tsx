import { useEffect, useRef, useState } from "react";
import { MicRecorder, listInputDevices } from "../audio/recorder";
import { fileToWav16k, samplesToWav16k, signalStats } from "../audio/wav";
import { deleteClip, readClip, saveClip } from "../speech/api";
import type { ClipInfo } from "../speech/types";

export interface Clip {
  id: string;
  label: string;
  source: "mic" | "import";
  info: ClipInfo;
  /** In-memory playback URL (the WAV file itself lives in the local clip store); null until loaded. */
  url: string | null;
}

interface Props {
  clips: Clip[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onAdd: (clip: Clip) => void;
  onRemove: (id: string) => void;
  onLoaded: (id: string, url: string) => void;
  onError: (message: string | null) => void;
}

const seconds = (ms: number) => `${(ms / 1000).toFixed(1)} s`;

export default function ClipPanel({ clips, selectedId, onSelect, onAdd, onRemove, onLoaded, onError }: Props) {
  const recorder = useRef(new MicRecorder());
  const timer = useRef<number | null>(null);
  const [recording, setRecording] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [level, setLevel] = useState(0);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [deviceId, setDeviceId] = useState("");

  const refreshDevices = () => listInputDevices().then(setDevices).catch(() => setDevices([]));
  useEffect(() => {
    void refreshDevices();
    return () => {
      if (timer.current) window.clearInterval(timer.current);
    };
  }, []);

  async function store(wav: Uint8Array, label: string, source: Clip["source"]) {
    const info = await saveClip(wav);
    const url = URL.createObjectURL(new Blob([wav as BlobPart], { type: "audio/wav" }));
    onAdd({ id: info.path, label, source, info, url });
  }

  async function startRecording() {
    onError(null);
    setNotice(null);
    try {
      await recorder.current.start(deviceId || undefined, setLevel);
      setRecording(true);
      setElapsed(0);
      const t0 = Date.now();
      timer.current = window.setInterval(() => setElapsed(Date.now() - t0), 200);
      void refreshDevices(); // labels become available once permission is granted
    } catch (e) {
      onError(`Cannot start recording: ${String(e)}`);
    }
  }

  async function stopRecording() {
    if (timer.current) window.clearInterval(timer.current);
    setRecording(false);
    setLevel(0);
    setBusy(true);
    try {
      const rec = await recorder.current.stop();
      if (rec.samples.length === 0) throw new Error("No audio was captured.");
      const stats = signalStats(rec.samples);
      const peakPct = Math.round(stats.peak * 100);
      if (stats.clippedRatio > 0.001) {
        setNotice(
          `Input is clipping (peak ${peakPct} %, ${(stats.clippedRatio * 100).toFixed(1)} % of samples at full scale). ` +
            "Lower the microphone volume in Windows (Settings, System, Sound, Input) or move away from the microphone, then record again.",
        );
      } else if (stats.peak < 0.05) {
        setNotice(`Input is very quiet (peak ${peakPct} %). Raise the microphone volume or move closer.`);
      } else {
        setNotice(`Input level OK (peak ${peakPct} %).`);
      }
      const { wav } = await samplesToWav16k(rec.samples, rec.sampleRate);
      const stamp = new Date().toLocaleTimeString();
      await store(wav, `Recording ${stamp}${rec.deviceLabel ? ` (${rec.deviceLabel})` : ""}`, "mic");
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function importFile(file: File | undefined) {
    if (!file) return;
    onError(null);
    setBusy(true);
    try {
      const { wav } = await fileToWav16k(file);
      await store(wav, file.name, "import");
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function load(clip: Clip) {
    try {
      const bytes = await readClip(clip.info.path);
      onLoaded(clip.id, URL.createObjectURL(new Blob([bytes], { type: "audio/wav" })));
    } catch (e) {
      onError(String(e));
    }
  }

  async function remove(clip: Clip) {
    try {
      await deleteClip(clip.info.path);
      if (clip.url) URL.revokeObjectURL(clip.url);
      onRemove(clip.id);
    } catch (e) {
      onError(String(e));
    }
  }

  return (
    <section>
      <strong>Audio</strong>
      <p className="hint">
        Recordings and imports are converted to 16 kHz mono WAV and stored only on this computer
        (delete them with the button below). Browser noise suppression, echo cancellation and gain
        control are switched off.
      </p>

      <label htmlFor="mic">Microphone</label>
      <select id="mic" value={deviceId} onChange={(e) => setDeviceId(e.target.value)} disabled={recording}>
        <option value="">System default</option>
        {devices.map((d, i) => (
          <option key={d.deviceId || i} value={d.deviceId}>
            {d.label || `Microphone ${i + 1}`}
          </option>
        ))}
      </select>

      {!recording ? (
        <button onClick={() => void startRecording()} disabled={busy}>● Record</button>
      ) : (
        <button onClick={() => void stopRecording()}>■ Stop ({seconds(elapsed)})</button>
      )}
      {recording && (
        <span className="meter" aria-label="input level">
          <span style={{ width: `${Math.min(100, Math.round(level * 400))}%` }} />
        </span>
      )}

      <label htmlFor="import">Import an audio file (WAV, MP3, M4A, Ogg/Opus, WebM...)</label>
      <input
        id="import"
        type="file"
        accept="audio/*,.wav,.mp3,.m4a,.ogg,.opus,.webm,.aac,.flac"
        disabled={busy || recording}
        onChange={(e) => {
          void importFile(e.target.files?.[0]);
          e.target.value = "";
        }}
      />
      {busy && <p className="hint">Processing audio…</p>}
      {notice && <p className={notice.startsWith("Input level OK") ? "hint" : "warn"}>{notice}</p>}

      {clips.length > 0 && (
        <table>
          <thead>
            <tr><th /><th>Clip</th><th>Length</th><th>Listen</th><th /></tr>
          </thead>
          <tbody>
            {clips.map((c) => (
              <tr key={c.id}>
                <td>
                  <input
                    type="radio"
                    name="clip"
                    checked={selectedId === c.id}
                    onChange={() => onSelect(c.id)}
                    aria-label={`Use ${c.label}`}
                  />
                </td>
                <td title={c.info.path}>{c.label}</td>
                <td>{seconds(c.info.durationMs)}</td>
                <td>
                  {c.url ? (
                    <audio controls src={c.url} preload="none" />
                  ) : (
                    <button onClick={() => void load(c)}>Load player</button>
                  )}
                </td>
                <td><button onClick={() => void remove(c)}>Delete</button></td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
