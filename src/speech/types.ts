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
  processingMs: number;
  audioMs: number | null;
  isMock: boolean;
}
