import { useCallback, useEffect, useState } from "react";
import {
  cancelModelDownload,
  cancelTranscription,
  installModel,
  listClips,
  listModels,
  listSttProviders,
  onDownloadProgress,
  transcribe,
} from "./speech/api";
import ClipPanel, { type Clip } from "./components/ClipPanel";
import ComparePanel from "./components/ComparePanel";
import type {
  DownloadProgress,
  ModelInfo,
  ProviderInfo,
  TranscribeResult,
} from "./speech/types";

const mb = (bytes: number) => `${Math.round(bytes / 1_000_000)} MB`;

export default function App() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [providerId, setProviderId] = useState("");
  const [modelId, setModelId] = useState("");
  const [language, setLanguage] = useState("fr");
  const [audioPath, setAudioPath] = useState("");
  const [clips, setClips] = useState<Clip[]>([]);
  const [clipId, setClipId] = useState<string | null>(null);
  const [result, setResult] = useState<TranscribeResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);

  const refreshModels = useCallback(
    () => listModels().then(setModels).catch((e) => setError(String(e))),
    [],
  );

  useEffect(() => {
    listSttProviders()
      .then((list) => {
        setProviders(list);
        if (list.length > 0) setProviderId(list[0].id);
      })
      .catch((e) => setError(`Cannot reach the Rust backend: ${String(e)}`));
    void refreshModels();
    // Recordings stay on disk between sessions: list them so they can be reused or deleted.
    listClips()
      .then((saved) =>
        setClips(
          saved.map((info) => {
            const stamp = Number(/clip-(\d+)\.wav$/.exec(info.path)?.[1]);
            return {
              id: info.path,
              label: Number.isFinite(stamp) ? `Saved clip ${new Date(stamp).toLocaleString()}` : "Saved clip",
              source: "import" as const,
              info,
              url: null,
            };
          }),
        ),
      )
      .catch((e) => setError(String(e)));
    const unlisten = onDownloadProgress(setProgress);
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [refreshModels]);

  const selected = providers.find((p) => p.id === providerId);
  const providerModels = models.filter(
    (m) => m.provider === providerId && m.languages.includes(language),
  );
  const selectedModel = providerModels.find((m) => m.id === modelId);

  // Keep the chosen model valid when the engine or language changes.
  useEffect(() => {
    if (!providerModels.some((m) => m.id === modelId)) {
      setModelId(providerModels.find((m) => m.installStatus === "installed")?.id ?? providerModels[0]?.id ?? "");
    }
  }, [providerModels, modelId]);

  function selectClip(id: string) {
    const clip = clips.find((c) => c.id === id);
    if (!clip) return;
    setClipId(id);
    setAudioPath(clip.info.path);
  }

  function removeClip(id: string) {
    setClips((cs) => cs.filter((c) => c.id !== id));
    if (clipId === id) {
      setClipId(null);
      setAudioPath("");
    }
  }

  async function install(id: string) {
    setInstalling(id);
    setError(null);
    setProgress(null);
    try {
      await installModel(id);
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(null);
      setProgress(null);
      void refreshModels();
    }
  }

  async function run() {
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      setResult(await transcribe({ providerId, modelId, language, audioPath }));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main>
      <h1>SpeechLab</h1>
      <p className="sub">Offline speech-to-text / text-to-speech evaluation (experimental, M4: microphone and engine comparison)</p>

      <section>
        <strong>Models</strong>
        <p className="hint">Models are downloaded once from their official source, then used fully offline.</p>
        <table>
          <thead>
            <tr><th>Model</th><th>Size</th><th>License</th><th>Status</th><th /></tr>
          </thead>
          <tbody>
            {models.map((m) => (
              <tr key={m.id}>
                <td title={`${m.architecture} · ${m.quantization} · ${m.runtime}`}>{m.displayName}</td>
                <td>{mb(m.sizeBytes)}</td>
                <td>{m.license}</td>
                <td>{m.installStatus === "installed" ? "installed" : "not installed"}</td>
                <td>
                  {m.installStatus === "notInstalled" && (
                    <button disabled={installing !== null} onClick={() => void install(m.id)}>
                      {installing === m.id ? "Installing…" : "Install"}
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {installing && progress && (
          <p>
            {progress.phase}: {mb(progress.doneBytes)} / {mb(progress.totalBytes)}{" "}
            <button onClick={() => void cancelModelDownload()}>Cancel download</button>
          </p>
        )}
      </section>

      <ClipPanel
        clips={clips}
        selectedId={clipId}
        onSelect={selectClip}
        onAdd={(clip) => {
          setClips((cs) => [...cs, clip]);
          setClipId(clip.id);
          setAudioPath(clip.info.path);
        }}
        onRemove={removeClip}
        onLoaded={(id, url) => setClips((cs) => cs.map((c) => (c.id === id ? { ...c, url } : c)))}
        onError={setError}
      />

      <section>
        <strong>Speech-to-Text</strong>
        <label htmlFor="provider">Engine</label>
        <select id="provider" value={providerId} onChange={(e) => setProviderId(e.target.value)}>
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.displayName}
              {p.isMock ? " (mock)" : ""}
            </option>
          ))}
        </select>

        {selected && (
          <p>
            Cancellation: {selected.capabilities.supportsCancellation ? "yes" : "no (only before a run starts)"} ·
            Auto language detection (available, not used): {selected.capabilities.supportsLanguageAutoDetect ? "yes" : "no"}
          </p>
        )}

        <label htmlFor="lang">Language (explicit)</label>
        <select id="lang" value={language} onChange={(e) => setLanguage(e.target.value)}>
          <option value="fr">French (fr)</option>
          <option value="en">English (en)</option>
        </select>

        <label htmlFor="model">Model</label>
        <select id="model" value={modelId} onChange={(e) => setModelId(e.target.value)}>
          {providerModels.map((m) => (
            <option key={m.id} value={m.id}>
              {m.displayName}
              {m.installStatus === "installed" ? "" : " — not installed"}
            </option>
          ))}
        </select>

        <label htmlFor="audio">Audio file path (the selected clip, or type the path of a WAV file)</label>
        <input
          id="audio"
          value={audioPath}
          onChange={(e) => {
            setAudioPath(e.target.value);
            setClipId(null);
          }}
          placeholder="C:\\path\\to\\sample.wav"
        />

        <button onClick={run} disabled={busy || !modelId || selectedModel?.installStatus !== "installed"}>
          {busy ? "Running…" : "Transcribe"}
        </button>
        <button onClick={() => void cancelTranscription()} disabled={!busy}>
          Cancel
        </button>

        {error && <div className="error">{error}</div>}
        {result?.isMock && (
          <div className="mock">Mock provider: no real recognition was performed. Do not use this output for evaluation.</div>
        )}
        {result && (
          <>
            <label htmlFor="out">Transcription (editable)</label>
            <textarea id="out" value={result.text} onChange={(e) => setResult({ ...result, text: e.target.value })} rows={5} />
            <small>
              {result.providerId} · {result.modelId} · {result.language} · inference {result.processingMs} ms
              {result.audioMs !== null && ` for ${result.audioMs} ms of audio`}
              {result.rtf !== null && ` (RTF ${result.rtf.toFixed(3)})`} ·{" "}
              {result.coldStart ? `cold start, model load ${result.loadMs} ms` : "warm (model already loaded)"} ·{" "}
              {result.threads} threads · decoding: {result.decoding}
            </small>
          </>
        )}
      </section>

      <ComparePanel models={models} language={language} audioPath={audioPath} />
    </main>
  );
}
