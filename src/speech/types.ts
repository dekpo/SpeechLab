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
  /** Terms to favour (drug names, technical terms) for engines that support it. Default: none. */
  vocabulary?: string[];
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
  /** Peak resident memory while the engine ran (sampled, a lower bound), MB. */
  peakMemoryMb: number | null;
  /** CPU time used by the engine, all threads, ms. */
  cpuMs: number | null;
  /** Decoding strategy actually used, e.g. "greedy search" or "beam search (5 beams)". */
  decoding: string;
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

export interface ClipInfo {
  path: string;
  durationMs: number;
  sampleRate: number;
  bytes: number;
}

export type DiffKind = "equal" | "substitute" | "delete" | "insert";

export interface DiffOp {
  kind: DiffKind;
  reference: string | null;
  hypothesis: string | null;
}

export interface TextComparison {
  /** null when the reference has no words. */
  wer: number | null;
  cer: number | null;
  referenceWords: number;
  hypothesisWords: number;
  substitutions: number;
  deletions: number;
  insertions: number;
  diff: DiffOp[];
  normalizedReference: string;
  normalizedHypothesis: string;
}

// ---------- Benchmark dataset (M5b) ----------

export interface KeyTerm {
  text: string;
  kind: string;
}

export interface ScriptItem {
  id: string;
  language: string;
  domain: string;
  utteranceType: string;
  category: string;
  text: string;
  keyTerms?: KeyTerm[];
}

export interface Speaker {
  id: string;
  /** Honest description of this ONE speaker. */
  profile: string;
  gender?: string | null;
  accent?: string | null;
}

export interface SourceInfo {
  kind: string;
  url?: string | null;
  license: string;
}

export interface Sample {
  id: string;
  language: string;
  domain: string;
  utteranceType: string;
  category: string;
  speaker: Speaker;
  audioFile: string;
  durationMs?: number | null;
  reference: string;
  source: SourceInfo;
  committable: boolean;
  private: boolean;
}

export interface ValidationIssue {
  sampleId: string;
  message: string;
}

export interface DatasetStatus {
  root: string;
  samples: Sample[];
  issues: ValidationIssue[];
}

// ---------- Text-to-speech (M6) ----------

export interface VoiceInfo {
  id: string;
  displayName: string;
  language: string;
  /** "female", "male" or "unknown": only what the voice's documentation states. */
  gender: string;
  license: string;
  modelId: string;
  speakerId: number;
  speakerCount: number;
}

export interface SynthesizeRequest {
  providerId: string;
  voiceId: string;
  language: string;
  text: string;
  /** 1.0 = normal speed. */
  speed: number;
}

export interface SynthesizeResult {
  providerId: string;
  voiceId: string;
  wavPath: string;
  generationMs: number;
  audioMs: number;
  loadMs: number;
  coldStart: boolean;
  sampleRate: number;
  rtf: number | null;
  speed: number;
  peakMemoryMb: number | null;
  isMock: boolean;
}
