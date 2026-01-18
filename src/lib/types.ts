export enum FileStatus {
  IDLE = "IDLE",
  CONVERTING = "CONVERTING",
  COMPLETED = "COMPLETED",
  ERROR = "ERROR",
}

export interface ConversionConfig {
  container: string;
  videoCodec: string;
  audioCodec: string;
  resolution: string;
  crf: number;
  preset: string;
}

export interface FileItem {
  id: string;
  name: string;
  size: number;
  status: FileStatus;
  progress: number;
  originalFormat: string;
  config: ConversionConfig;
  outputName: string;
  metadata?: SourceMetadata;
  metadataStatus: MetadataStatus;
  metadataError?: string;
  path: string;
}

export interface PresetDefinition {
  id: string;
  name: string;
  config: ConversionConfig;
  builtIn?: boolean;
}

export interface SourceMetadata {
  duration?: string;
  bitrate?: string;
  videoCodec?: string;
  audioCodec?: string;
  resolution?: string;
}

export type MetadataStatus = "idle" | "loading" | "ready" | "error";

export interface OutputEstimate {
  videoKbps: number;
  audioKbps: number;
  totalKbps: number;
  sizeMb?: number;
}
