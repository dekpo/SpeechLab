import { invoke } from "@tauri-apps/api/core";
import type { ProviderInfo, TranscribeRequest, TranscribeResult } from "./types";

// Thin wrappers over the Tauri commands. No engine-specific code lives here.

export const listSttProviders = (): Promise<ProviderInfo[]> =>
  invoke<ProviderInfo[]>("list_stt_providers");

export const transcribe = (request: TranscribeRequest): Promise<TranscribeResult> =>
  invoke<TranscribeResult>("transcribe", { request });

export const cancelTranscription = (): Promise<void> => invoke<void>("cancel_transcription");
