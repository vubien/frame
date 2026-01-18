import { invoke } from "@tauri-apps/api/core";
import type { ConversionConfig, OutputEstimate, SourceMetadata } from "$lib/types";

export async function probeMedia(filePath: string): Promise<SourceMetadata> {
  return invoke("probe_media", { filePath });
}

export async function estimateOutput(
  config: ConversionConfig,
  metadata?: SourceMetadata,
): Promise<OutputEstimate> {
  return invoke("estimate_output", { config, metadata });
}
