import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DownloadProgress,
  ModelInfo,
  ProviderInfo,
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
