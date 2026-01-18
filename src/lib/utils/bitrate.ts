import type { ConversionConfig, SourceMetadata } from "$lib/types";

const RESOLUTION_HEIGHTS: Record<string, number> = {
  "480p": 480,
  "720p": 720,
  "1080p": 1080,
};

const BASE_BITRATES: Array<{ height: number; kbps: number }> = [
  { height: 2160, kbps: 25000 },
  { height: 1440, kbps: 16000 },
  { height: 1080, kbps: 8000 },
  { height: 720, kbps: 5000 },
  { height: 480, kbps: 2500 },
  { height: 360, kbps: 1500 },
];

const AUDIO_BITRATES: Record<string, number> = {
  aac: 128,
  ac3: 192,
  libopus: 96,
  mp3: 128,
};

const CODEC_SCALE: Record<string, number> = {
  libx264: 1,
  "h264": 1,
  libx265: 0.65,
  "h265": 0.65,
  vp9: 0.7,
  "libvpx-vp9": 0.7,
  prores: 1.6,
};

export function parseDurationToSeconds(duration?: string): number | null {
  if (!duration) return null;
  const match = duration.match(/(\d{2}):(\d{2}):(\d{2})\.(\d{2})/);
  if (!match) return null;
  const [, hh, mm, ss, cs] = match;
  const hours = parseInt(hh, 10);
  const minutes = parseInt(mm, 10);
  const seconds = parseInt(ss, 10);
  const centiseconds = parseInt(cs, 10);
  return hours * 3600 + minutes * 60 + seconds + centiseconds / 100;
}

function parseResolutionHeight(resolution?: string): number | null {
  if (!resolution) return null;
  const match = resolution.match(/(\d{2,5})x(\d{2,5})/i);
  if (!match) return null;
  const [, , height] = match;
  return parseInt(height, 10);
}

function inferTargetHeight(config: ConversionConfig, metadata?: SourceMetadata): number {
  if (config.resolution !== "original") {
    return RESOLUTION_HEIGHTS[config.resolution] ?? 720;
  }
  const metaHeight = parseResolutionHeight(metadata?.resolution);
  return metaHeight ?? 720;
}

function baseVideoBitrate(height: number): number {
  const tier = BASE_BITRATES.find((t) => height >= t.height);
  return tier ? tier.kbps : BASE_BITRATES[BASE_BITRATES.length - 1].kbps;
}

function codecScaleFactor(codec: string): number {
  const key = codec.toLowerCase();
  return CODEC_SCALE[key] ?? 1;
}

function parseSourceBitrate(metadata?: SourceMetadata): number | null {
  const raw = metadata?.bitrate;
  if (!raw) return null;
  const match = raw.replace(/,/g, "").match(/([\d.]+)/);
  if (!match) return null;
  const value = parseFloat(match[1]);
  if (Number.isNaN(value)) return null;
  // Assume metadata bitrate already in kb/s
  return value;
}

function crfScale(crf: number): number {
  const diff = 23 - crf;
  return Math.pow(2, diff / 6);
}

function audioBitrate(codec: string): number {
  return AUDIO_BITRATES[codec.toLowerCase()] ?? 128;
}

export interface OutputEstimate {
  videoKbps: number;
  audioKbps: number;
  totalKbps: number;
  sizeMb?: number;
}

export function estimateOutput(config: ConversionConfig, metadata?: SourceMetadata): OutputEstimate {
  const height = inferTargetHeight(config, metadata);
  const sourceVideoBitrate = parseSourceBitrate(metadata);

  let videoKbps =
    sourceVideoBitrate ??
    baseVideoBitrate(height) * codecScaleFactor(config.videoCodec || "");

  videoKbps *= crfScale(config.crf);
  videoKbps = Math.max(400, videoKbps);

  const audioKbps = audioBitrate(config.audioCodec);
  const totalKbps = videoKbps + audioKbps;

  const durationSeconds = parseDurationToSeconds(metadata?.duration);
  const sizeMb =
    durationSeconds && totalKbps
      ? (totalKbps * durationSeconds) / 8 / 1024
      : undefined;

  return {
    videoKbps: Math.round(videoKbps),
    audioKbps: Math.round(audioKbps),
    totalKbps: Math.round(totalKbps),
    sizeMb: sizeMb ? Math.max(sizeMb, 1) : undefined,
  };
}

export function formatFileSize(sizeMb?: number): string {
  if (!sizeMb) return "—";
  if (sizeMb >= 1024) {
    return `${(sizeMb / 1024).toFixed(1)} GB`;
  }
  return `${sizeMb.toFixed(1)} MB`;
}
