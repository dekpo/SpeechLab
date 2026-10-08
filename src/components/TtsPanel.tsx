import { useCallback, useEffect, useRef, useState } from "react";
import {
  cancelModelDownload,
  cancelSynthesis,
  clearTtsAudio,
  installModel,
  listTtsModels,
  listTtsProviders,
  listTtsVoices,
  onDownloadProgress,
  readTtsAudio,
  synthesize,
} from "../speech/api";
import type {
  DownloadProgress,
  ModelInfo,
  ProviderInfo,
  SynthesizeResult,
  VoiceInfo,
} from "../speech/types";

const mb = (bytes: number) => `${Math.round(bytes / 1_000_000)} MB`;

/** What each commercial-use rating means, in plain words (evidence: docs/TTS_LICENSES.md). */
const TIER_TEXT: Record<string, string> = {
  clear: "clear: public domain or CC0, trained from scratch",
  attribution: "attribution: credit required, commercial use allowed",
  review: "REVIEW: legal reading needed before commercial use",
  excluded: "EXCLUDED: not usable commercially as it stands",
  unrated: "not rated",
};
const tierText = (tier: string) => TIER_TEXT[tier] ?? tier;

const SAMPLE_TEXT: Record<string, string> = {
  fr: "Le patient prend cinq cents milligrammes d'amoxicilline trois fois par jour pendant sept jours.",
  en: "The patient takes five hundred milligrams of amoxicillin three times a day for seven days.",
};

interface Props {
  onError: (message: string | null) => void;
}

/**
 * Text-to-speech laboratory. Engine-agnostic: it talks to the generic TTS commands only. It never
 * plays audio by itself: "Generate" writes a file, the player below it is controlled by the user.
 */
export default function TtsPanel({ onError }: Props) {
  const [provider, setProvider] = useState<ProviderInfo | null>(null);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [voices, setVoices] = useState<VoiceInfo[]>([]);
  const [language, setLanguage] = useState("fr");
  const [voiceId, setVoiceId] = useState("");
  const [speed, setSpeed] = useState(1);
  const [text, setText] = useState(SAMPLE_TEXT.fr);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<SynthesizeResult | null>(null);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [installing, setInstalling] = useState<string | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const player = useRef<HTMLAudioElement>(null);

  const refresh = useCallback(async (providerId: string) => {
    try {
      setModels(await listTtsModels());
      setVoices(await listTtsVoices(providerId));
    } catch (e) {
      onError(String(e));
    }
  }, [onError]);

  useEffect(() => {
    listTtsProviders()
      .then((list) => {
        if (list.length > 0) {
          setProvider(list[0]);
          void refresh(list[0].id);
        }
      })
      .catch((e) => onError(String(e)));
    const unlisten = onDownloadProgress(setProgress);
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [refresh, onError]);

  // Free the audio object URL when it is replaced or the panel goes away.
  useEffect(() => () => { if (audioUrl) URL.revokeObjectURL(audioUrl); }, [audioUrl]);

  const languageVoices = voices.filter((v) => v.language === language);
  const voice = voices.find((v) => v.id === voiceId);
  useEffect(() => {
    if (!languageVoices.some((v) => v.id === voiceId)) setVoiceId(languageVoices[0]?.id ?? "");
  }, [languageVoices, voiceId]);

  function changeLanguage(next: string) {
    // Replace the text only if it is still the untouched sample of the other language.
    if (Object.values(SAMPLE_TEXT).includes(text)) setText(SAMPLE_TEXT[next] ?? text);
    setLanguage(next);
  }

  async function install(id: string) {
    setInstalling(id);
    setProgress(null);
    onError(null);
    try {
      await installModel(id);
    } catch (e) {
      onError(String(e));
    } finally {
      setInstalling(null);
      setProgress(null);
      if (provider) void refresh(provider.id);
    }
  }

  async function generate() {
    if (!provider || !voice) return;
    setBusy(true);
    onError(null);
    setResult(null);
    setAudioUrl(null);
    try {
      const r = await synthesize({ providerId: provider.id, voiceId: voice.id, language, text, speed });
      const bytes = await readTtsAudio(r.wavPath);
      setAudioUrl(URL.createObjectURL(new Blob([bytes], { type: "audio/wav" })));
      setResult(r);
    } catch (e) {
      // A cancellation the user asked for is not an error.
      if (!String(e).includes("operation cancelled")) onError(String(e));
    } finally {
      setBusy(false);
    }
  }

  function stop() {
    const el = player.current;
    if (el) {
      el.pause();
      el.currentTime = 0;
    }
  }

  async function clearFiles() {
    try {
      const n = await clearTtsAudio();
      setResult(null);
      setAudioUrl(null);
      onError(null);
      window.alert(`${n} generated audio file(s) deleted.`);
    } catch (e) {
      onError(String(e));
    }
  }

  return (
    <section>
      <strong>Text-to-Speech</strong>
      <p className="hint">
        Voices run locally (CPU). Nothing is played automatically: generate first, then press play.
        Gender is shown only when the voice documentation states it. The phonemizer inside these packages
        is espeak-ng (GPL-3.0), a licensing point to settle before any product use (D-012).
      </p>

      <table>
        <thead>
          <tr><th>Voice package</th><th>Languages</th><th>Size</th><th>License</th><th>Commercial use</th><th>Status</th><th /></tr>
        </thead>
        <tbody>
          {models.map((m) => (
            <tr key={m.id}>
              <td title={`${m.architecture} · ${m.quantization} · ${m.runtime}`}>{m.displayName}</td>
              <td>{m.languages.join(", ")}</td>
              <td>{mb(m.sizeBytes)}</td>
              <td>{m.license}</td>
              <td title={m.licenseNotes}>
                <span className={`tier tier-${m.licenseTier}`}>{tierText(m.licenseTier)}</span>
              </td>
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

      <label htmlFor="tts-lang">Language</label>
      <select id="tts-lang" value={language} onChange={(e) => changeLanguage(e.target.value)}>
        <option value="fr">French (fr)</option>
        <option value="en">English (en)</option>
      </select>

      <label htmlFor="tts-voice">Voice</label>
      <select id="tts-voice" value={voiceId} onChange={(e) => setVoiceId(e.target.value)} disabled={languageVoices.length === 0}>
        {languageVoices.map((v) => (
          <option key={v.id} value={v.id}>
            {v.displayName} · {v.gender} · [{v.licenseTier}]
          </option>
        ))}
      </select>
      {languageVoices.length === 0 && <p className="hint">No installed voice for this language: install a package above.</p>}
      {voice && (
        <p className="hint">
          Package {voice.modelId} · speaker {voice.speakerId} of {voice.speakerCount} · license: {voice.license}
        </p>
      )}
      {voice && (voice.licenseTier === "review" || voice.licenseTier === "excluded") && (
        <div className="mock">
          Commercial use of this voice: {tierText(voice.licenseTier)}. Hover the rating in the table above for the reason.
          Fine for evaluation, not a cleared voice.
        </div>
      )}

      <label htmlFor="tts-speed">
        Speed {speed.toFixed(2)}
        {provider && !provider.capabilities.supportsSpeed && " (not supported by this engine)"}
      </label>
      <input
        id="tts-speed"
        type="range"
        min={0.5}
        max={2}
        step={0.05}
        value={speed}
        disabled={!provider?.capabilities.supportsSpeed}
        onChange={(e) => setSpeed(Number(e.target.value))}
      />
      <small>The effect on duration differs between voices: check the audio duration shown after each generation.</small>

      <label htmlFor="tts-text">Text</label>
      <textarea id="tts-text" value={text} onChange={(e) => setText(e.target.value)} rows={4} maxLength={5000} />

      <button onClick={() => void generate()} disabled={busy || !voice || !text.trim()}>
        {busy ? "Generating…" : "Generate"}
      </button>
      <button onClick={() => void cancelSynthesis()} disabled={!busy}>
        Cancel
      </button>

      {result && audioUrl && (
        <>
          <p>
            <audio ref={player} controls src={audioUrl} preload="metadata" />
          </p>
          <button onClick={stop}>Stop</button>{" "}
          <a href={audioUrl} download={`speechlab-tts-${language}.wav`}>
            <button type="button">Save WAV</button>
          </a>{" "}
          <button onClick={() => void clearFiles()}>Delete generated files</button>
          <p>
            <small>
              generation {result.generationMs} ms for {result.audioMs} ms of audio
              {result.rtf !== null && ` (RTF ${result.rtf.toFixed(3)})`} · {result.sampleRate} Hz ·{" "}
              {result.coldStart ? `cold start, voice load ${result.loadMs} ms` : "warm (voice already loaded)"} · speed{" "}
              {result.speed.toFixed(2)}
              {result.peakMemoryMb !== null && ` · peak memory ${Math.round(result.peakMemoryMb)} MB`}
            </small>
          </p>
        </>
      )}
    </section>
  );
}
