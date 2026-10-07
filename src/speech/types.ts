// Engine-agnostic contract. Mirrors src-tauri/src/speech/types.rs (camelCase over JSON).
// The UI must only depend on these types, never on an engine-specific API.

export type ProviderKind = "stt" | "tts";

export interface Capabilities {
  supportsCancellation: boolean;
  supportsLanguageAutoDetect: boolean;
  supportsStreaming: boolean;
  supportsSpeed: boolean;
}

export interface ProviderInfo {
  id: string;
  displayName: string;
  kind: ProviderKind;
  /** True for the placeholder provider; its output is never benchmark data. */
  isMock: boolean;
  languages: string[];
  capabilities: Capabilities;
}

export interface TranscribeRequest {
  providerId: string;
  modelId: string;
  /** Explicit language code, e.g. "fr" or "en". */
  language: string;
  /** Path of a WAV file on disk. */
  audioPath: string;
}

export interface TranscribeResult {
  providerId: string;
  modelId: string;
  language: string;
  text: string;
  /** Inference time only, excluding model loading. */
  processingMs: number;
  /** Model loading time; 0 when the model was already loaded. */
  loadMs: number;
  coldStart: boolean;
  audioMs: number | null;
  /** processingMs / audioMs; below 1 is faster than real time. */
  rtf: number | null;
  threads: number;
  isMock: boolean;
}

export type InstallStatus = "notInstalled" | "installed";

export interface ModelInfo {
  id: string;
  displayName: string;
  provider: string;
  languages: string[];
  architecture: string;
  quantization: string;
  sizeBytes: number;
  license: string;
  sourceUrl: string;
  runtime: string;
  platforms: string[];
  /** null = not measured yet. */
  expectedMemoryMb: number | null;
  installStatus: InstallStatus;
  installedPath: string | null;
}

export type DownloadPhase = "download" | "verify" | "extract" | "done";

export interface DownloadProgress {
  modelId: string;
  phase: DownloadPhase;
  doneBytes: number;
  totalBytes: number;
}
