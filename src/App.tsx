import { useEffect, useState } from "react";
import { cancelTranscription, listSttProviders, transcribe } from "./speech/api";
import type { ProviderInfo, TranscribeResult } from "./speech/types";

export default function App() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [providerId, setProviderId] = useState("");
  const [language, setLanguage] = useState("fr");
  const [audioPath, setAudioPath] = useState("");
  const [result, setResult] = useState<TranscribeResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    listSttProviders()
      .then((list) => {
        setProviders(list);
        if (list.length > 0) setProviderId(list[0].id);
      })
      .catch((e) => setError(`Cannot reach the Rust backend: ${String(e)}`));
  }, []);

  const selected = providers.find((p) => p.id === providerId);

  async function run() {
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      setResult(
        await transcribe({ providerId, modelId: "none", language, audioPath }),
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main>
      <h1>SpeechLab</h1>
      <p className="sub">Offline speech-to-text / text-to-speech evaluation (experimental, M1 skeleton)</p>

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
            Languages: {selected.languages.join(", ")} · Cancellation:{" "}
            {selected.capabilities.supportsCancellation ? "yes" : "no"} · Auto language detection:{" "}
            {selected.capabilities.supportsLanguageAutoDetect ? "yes" : "no"}
          </p>
        )}

        <label htmlFor="lang">Language (explicit)</label>
        <select id="lang" value={language} onChange={(e) => setLanguage(e.target.value)}>
          <option value="fr">French (fr)</option>
          <option value="en">English (en)</option>
        </select>

        <label htmlFor="audio">WAV file path</label>
        <input id="audio" value={audioPath} onChange={(e) => setAudioPath(e.target.value)} placeholder="C:\path\to\sample.wav" />

        <button onClick={run} disabled={busy || !providerId}>
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
            <pre>{result.text}</pre>
            <small>
              {result.providerId} · model {result.modelId} · {result.language} · {result.processingMs} ms
            </small>
          </>
        )}
      </section>
    </main>
  );
}
