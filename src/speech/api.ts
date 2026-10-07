import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ClipInfo,
  DatasetStatus,
  Sample,
  ScriptItem,
  SourceInfo,
  Speaker,
  DownloadProgress,
  ModelInfo,
  ProviderInfo,
  TextComparison,
  TranscribeRequest,
  TranscribeResult,
} from "./types";

// Thin wrappers over the Tauri commands. No engine-specific code lives here.

export const listSttProviders = (): Promise<ProviderInfo[]> =>
  invoke<ProviderInfo[]>("list_stt_providers");

export const listModels = (): Promise<ModelInfo[]> => invoke<ModelInfo[]>("list_models");

export const transcribe = (request: TranscribeRequest): Promise<TranscribeResult> =>
  invoke<TranscribeResult>("transcribe", { request });

export const cancelTranscription = (): Promise<void> => invoke<void>("cancel_transcription");

/** Downloads a model (the only network operation). Resolves with the archive SHA-256. */
export const installModel = (modelId: string): Promise<string> =>
  invoke<string>("install_model", { modelId });

export const cancelModelDownload = (): Promise<void> => invoke<void>("cancel_model_download");

export const onDownloadProgress = (
  handler: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> =>
  listen<DownloadProgress>("model-download-progress", (e) => handler(e.payload));

/** Saves a WAV clip locally (raw binary body, no JSON overhead). */
export const saveClip = (wav: Uint8Array): Promise<ClipInfo> => invoke<ClipInfo>("save_clip", wav);

export const deleteClip = (path: string): Promise<void> => invoke<void>("delete_clip", { path });

export const compareTexts = (reference: string, hypothesis: string): Promise<TextComparison> =>
  invoke<TextComparison>("compare_texts", { reference, hypothesis });

/** Clips already on disk (survive a UI reload). */
export const listClips = (): Promise<ClipInfo[]> => invoke<ClipInfo[]>("list_clips");

/** WAV bytes of a stored clip, for playback. */
export const readClip = (path: string): Promise<ArrayBuffer> => invoke<ArrayBuffer>("read_clip", { path });

// ---------- Benchmark dataset recorder ----------

export const listScripts = (): Promise<ScriptItem[]> => invoke<ScriptItem[]>("list_scripts");

export const listSamples = (): Promise<DatasetStatus> => invoke<DatasetStatus>("list_samples");

/** Turns a stored clip into a benchmark sample for a script sentence (reference = script text). */
export const createSample = (args: {
  scriptId: string;
  speaker: Speaker;
  source: SourceInfo;
  private: boolean;
  clipPath: string;
}): Promise<Sample> => invoke<Sample>("create_sample", args);

export const deleteSample = (sampleId: string): Promise<void> => invoke<void>("delete_sample", { sampleId });

export const readSampleAudio = (sampleId: string): Promise<ArrayBuffer> =>
  invoke<ArrayBuffer>("read_sample_audio", { sampleId });
