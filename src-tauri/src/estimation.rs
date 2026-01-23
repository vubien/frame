use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::command;

use crate::conversion::{ConversionConfig, ConversionError};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub index: u32,
    pub codec: String,
    pub channels: String,
    pub language: Option<String>,
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate_kbps: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProbeMetadata {
    pub duration: Option<String>,
    pub bitrate: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_bitrate_kbps: Option<f64>,
    pub audio_tracks: Vec<AudioTrack>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEstimate {
    pub video_kbps: u32,
    pub audio_kbps: u32,
    pub total_kbps: u32,
    pub size_mb: Option<f64>,
}

const FALLBACK_AUDIO_BITRATE_KBPS: f64 = 128.0;
const CONTAINER_CONTENT_RATIO: f64 = 0.95; // Assumes ~5% overhead
const DEFAULT_H264_BITS_PER_PIXEL: f64 = 0.075;
const DEFAULT_H265_BITS_PER_PIXEL: f64 = 0.040;
const DEFAULT_VP9_BITS_PER_PIXEL: f64 = 0.050;
const DEFAULT_AV1_BITS_PER_PIXEL: f64 = 0.032;
const DEFAULT_PRORES_BITS_PER_PIXEL: f64 = 1.9;

pub(crate) fn parse_frame_rate_string(value: Option<&str>) -> Option<f64> {
    let value = value?.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("n/a") {
        return None;
    }

    if let Some((num, den)) = value.split_once('/') {
        let numerator: f64 = num.trim().parse().ok()?;
        let denominator: f64 = den.trim().parse().ok()?;
        if denominator == 0.0 {
            return None;
        }
        Some(numerator / denominator)
    } else {
        value.parse::<f64>().ok()
    }
}

pub(crate) fn parse_probe_bitrate(raw: Option<&str>) -> Option<f64> {
    let raw = raw?.trim();
    if raw.eq_ignore_ascii_case("n/a") || raw.is_empty() {
        return None;
    }
    let numeric = raw.parse::<f64>().ok()?;
    if numeric <= 0.0 {
        return None;
    }
    Some(numeric / 1000.0)
}

pub(crate) fn is_audio_only_container(container: &str) -> bool {
    matches!(
        container.to_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "aac" | "m4a"
    )
}

fn parse_duration_to_seconds(duration: Option<&String>) -> Option<f64> {
    let duration_str = duration?;
    if let Ok(seconds) = duration_str.parse::<f64>() {
        return Some(seconds);
    }

    let parts: Vec<&str> = duration_str.split(':').collect();
    if parts.len() == 3 {
        let hours: f64 = parts[0].trim().parse().ok()?;
        let minutes: f64 = parts[1].trim().parse().ok()?;
        let seconds: f64 = parts[2].trim().parse().ok()?;
        return Some(hours * 3600.0 + minutes * 60.0 + seconds);
    }

    None
}

fn parse_resolution(metadata_resolution: Option<&String>) -> Option<(u32, u32)> {
    let resolution = metadata_resolution?;
    let parts: Vec<&str> = resolution.split('x').collect();
    if parts.len() != 2 {
        return None;
    }
    let width = parts[0].parse::<u32>().ok()?;
    let height = parts[1].parse::<u32>().ok()?;
    Some((width, height))
}

#[derive(Clone, Copy)]
struct VideoDimensions {
    width: u32,
    height: u32,
}

impl VideoDimensions {
    fn pixel_rate(&self, fps: f64) -> f64 {
        self.width as f64 * self.height as f64 * fps
    }
}

const DEFAULT_DIMENSIONS: VideoDimensions = VideoDimensions {
    width: 1280,
    height: 720,
};

const DEFAULT_ASPECT_RATIO: f64 = 16.0 / 9.0;
const DEFAULT_FRAME_RATE: f64 = 30.0;

fn metadata_dimensions(metadata: Option<&ProbeMetadata>) -> Option<VideoDimensions> {
    metadata.and_then(|meta| {
        if let (Some(w), Some(h)) = (meta.width, meta.height) {
            if w > 0 && h > 0 {
                return Some(VideoDimensions {
                    width: w,
                    height: h,
                });
            }
        }
        parse_resolution(meta.resolution.as_ref()).map(|(w, h)| VideoDimensions {
            width: w,
            height: h,
        })
    })
}

fn nominal_width_for_height(height: u32) -> u32 {
    match height {
        2160 => 3840,
        1440 => 2560,
        1080 => 1920,
        720 => 1280,
        576 => 1024,
        480 => 854,
        _ => (height as f64 * DEFAULT_ASPECT_RATIO).round() as u32,
    }
}

fn derive_dimension_from_aspect(known: u32, aspect: f64, is_width_known: bool) -> VideoDimensions {
    if is_width_known {
        let computed_height = (known as f64 / aspect).round().max(1.0) as u32;
        VideoDimensions {
            width: known,
            height: computed_height,
        }
    } else {
        let computed_width = (known as f64 * aspect).round().max(1.0) as u32;
        VideoDimensions {
            width: computed_width,
            height: known,
        }
    }
}

fn determine_target_dimensions(
    config: &ConversionConfig,
    metadata: Option<&ProbeMetadata>,
) -> Result<VideoDimensions, ConversionError> {
    let source_dimensions = metadata_dimensions(metadata);
    match config.resolution.as_str() {
        "original" => Ok(source_dimensions.unwrap_or(DEFAULT_DIMENSIONS)),
        "1080p" => Ok(resolution_from_height(1080, source_dimensions)),
        "720p" => Ok(resolution_from_height(720, source_dimensions)),
        "480p" => Ok(resolution_from_height(480, source_dimensions)),
        "custom" => custom_dimensions(config, source_dimensions),
        _ => Ok(source_dimensions.unwrap_or(DEFAULT_DIMENSIONS)),
    }
}

fn resolution_from_height(height: u32, source: Option<VideoDimensions>) -> VideoDimensions {
    let width = if height == 0 {
        DEFAULT_DIMENSIONS.width
    } else if let Some(dim) = source {
        (height as f64 * (dim.width as f64 / dim.height as f64))
            .round()
            .max(1.0) as u32
    } else {
        nominal_width_for_height(height)
    };

    VideoDimensions {
        width,
        height: height.max(1),
    }
}

fn custom_dimensions(
    config: &ConversionConfig,
    source: Option<VideoDimensions>,
) -> Result<VideoDimensions, ConversionError> {
    let parsed_width = parse_custom_dimension(config.custom_width.as_deref(), "customWidth")?;
    let parsed_height = parse_custom_dimension(config.custom_height.as_deref(), "customHeight")?;

    match (parsed_width, parsed_height) {
        (Some(w), Some(h)) => Ok(VideoDimensions {
            width: w,
            height: h,
        }),
        (Some(w), None) => {
            let aspect = source
                .map(|dim| dim.width as f64 / dim.height as f64)
                .unwrap_or(DEFAULT_ASPECT_RATIO);
            Ok(derive_dimension_from_aspect(w, aspect, true))
        }
        (None, Some(h)) => {
            let aspect = source
                .map(|dim| dim.width as f64 / dim.height as f64)
                .unwrap_or(DEFAULT_ASPECT_RATIO);
            Ok(derive_dimension_from_aspect(h, aspect, false))
        }
        (None, None) => Ok(source.unwrap_or(DEFAULT_DIMENSIONS)),
    }
}

fn parse_custom_dimension(
    value: Option<&str>,
    field: &str,
) -> Result<Option<u32>, ConversionError> {
    match value {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed == "-1" {
                return Ok(None);
            }
            let parsed = trimmed.parse::<i32>().map_err(|_| {
                ConversionError::InvalidInput(format!("Invalid {} value: {}", field, raw))
            })?;
            if parsed <= 0 {
                return Err(ConversionError::InvalidInput(format!(
                    "{} must be positive or -1, got {}",
                    field, raw
                )));
            }
            Ok(Some(parsed as u32))
        }
    }
}

fn determine_target_fps(
    config: &ConversionConfig,
    metadata: Option<&ProbeMetadata>,
) -> Result<f64, ConversionError> {
    if config.fps == "original" {
        return Ok(metadata
            .and_then(|meta| meta.frame_rate)
            .unwrap_or(DEFAULT_FRAME_RATE));
    }

    let parsed =
        config.fps.trim().parse::<f64>().map_err(|_| {
            ConversionError::InvalidInput(format!("Invalid fps value: {}", config.fps))
        })?;
    if parsed <= 0.0 {
        return Err(ConversionError::InvalidInput(format!(
            "Frame rate must be positive, got {}",
            config.fps
        )));
    }
    Ok(parsed)
}

fn parse_config_bitrate(value: &str, field: &str) -> Result<f64, ConversionError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConversionError::InvalidInput(format!(
            "{} must not be empty",
            field
        )));
    }
    let normalized = trimmed.replace(',', ".");
    let parsed = normalized.parse::<f64>().map_err(|_| {
        ConversionError::InvalidInput(format!("Invalid {} value: {}", field, value))
    })?;
    if parsed <= 0.0 {
        return Err(ConversionError::InvalidInput(format!(
            "{} must be positive, got {}",
            field, value
        )));
    }
    Ok(parsed)
}

fn resolve_audio_track_ids(
    config: &ConversionConfig,
    metadata: Option<&ProbeMetadata>,
    audio_only: bool,
) -> Vec<u32> {
    if !config.selected_audio_tracks.is_empty() {
        let mut seen = HashSet::new();
        let mut ordered = Vec::new();
        for id in &config.selected_audio_tracks {
            if seen.insert(*id) {
                ordered.push(*id);
            }
        }
        return ordered;
    }

    if let Some(meta) = metadata {
        if let Some(first) = meta.audio_tracks.first() {
            return vec![first.index];
        }
    }

    if audio_only {
        return vec![0];
    }

    Vec::new()
}

fn estimate_audio_bitrate_kbps(
    config: &ConversionConfig,
    metadata: Option<&ProbeMetadata>,
    audio_only: bool,
) -> Result<f64, ConversionError> {
    let track_ids = resolve_audio_track_ids(config, metadata, audio_only);
    if track_ids.is_empty() {
        return Ok(0.0);
    }

    if config.audio_codec.eq_ignore_ascii_case("copy") {
        if let Some(meta) = metadata {
            let mut sum = 0.0;
            for id in &track_ids {
                if let Some(track) = meta.audio_tracks.iter().find(|t| t.index == *id) {
                    if let Some(br) = track.bitrate_kbps {
                        sum += br;
                    }
                }
            }
            if sum > 0.0 {
                return Ok(sum);
            }
            return Ok(FALLBACK_AUDIO_BITRATE_KBPS * track_ids.len() as f64);
        }
    }

    // Handle PCM/uncompressed audio gracefully (default to 1536 kbps if 0/invalid)
    if config.audio_codec.to_lowercase().starts_with("pcm_") {
        let parsed = config.audio_bitrate.parse::<f64>().unwrap_or(0.0);
        let bitrate = if parsed > 0.0 { parsed } else { 1536.0 };
        return Ok(bitrate * track_ids.len() as f64);
    }

    let per_track = parse_config_bitrate(&config.audio_bitrate, "audio bitrate")?;
    Ok(per_track * track_ids.len() as f64)
}

fn container_overhead_factor(container: &str) -> f64 {
    match container.to_lowercase().as_str() {
        "ts" | "m2ts" => 1.04,
        "mkv" | "webm" => 1.01,
        "mp4" | "m4v" | "mov" => 1.008,
        _ => 1.01,
    }
}

fn estimate_quality_video_bitrate(
    config: &ConversionConfig,
    metadata: Option<&ProbeMetadata>,
    target_dimensions: VideoDimensions,
    target_fps: f64,
) -> f64 {
    let source_dimensions = metadata_dimensions(metadata).unwrap_or(target_dimensions);
    let source_fps = metadata
        .and_then(|meta| meta.frame_rate)
        .filter(|fps| *fps > 0.0)
        .unwrap_or(target_fps);
    let target_pixel_rate = target_dimensions.pixel_rate(target_fps);
    let source_pixel_rate = source_dimensions.pixel_rate(source_fps).max(1.0);
    let pixel_ratio = target_pixel_rate / source_pixel_rate;

    let is_hardware = config.video_codec.contains("videotoolbox") || config.video_codec.contains("nvenc");
    let effective_crf = if is_hardware {
        (100.0 - config.quality as f64) / 1.96
    } else {
        config.crf as f64
    };

    if let Some(source_bitrate) = source_video_bitrate_kbps(metadata) {
        let reference = codec_reference(&config.video_codec);
        let source_bits_per_pixel = (source_bitrate * 1000.0) / source_pixel_rate;

        if source_bits_per_pixel > 0.0 && reference.reference_bits_per_pixel > 0.0 {
            let quality_ratio = source_bits_per_pixel / reference.reference_bits_per_pixel;
            if quality_ratio.is_finite() && quality_ratio > 0.0 {
                let source_crf = reference.reference_crf - 6.0 * quality_ratio.log2();
                let quality_factor = 2f64.powf((source_crf - effective_crf) / 6.0);
                return (source_bitrate * pixel_ratio * quality_factor).max(0.0);
            }
        }

        return (source_bitrate * pixel_ratio).max(0.0);
    }

    reference_bitrate_from_quality(&config.video_codec, effective_crf, target_pixel_rate)
}

fn source_video_bitrate_kbps(metadata: Option<&ProbeMetadata>) -> Option<f64> {
    metadata.and_then(|meta| {
        if let Some(video_kbps) = meta.video_bitrate_kbps {
            if video_kbps > 0.0 {
                return Some(video_kbps);
            }
        }
        parse_probe_bitrate(meta.bitrate.as_deref()).map(|container_kbps| {
            let audio_sum: f64 = meta
                .audio_tracks
                .iter()
                .filter_map(|track| track.bitrate_kbps)
                .sum();
            let effective_container_kbps = container_kbps * CONTAINER_CONTENT_RATIO;
            (effective_container_kbps - audio_sum).max(0.0)
        })
    })
}

#[derive(Clone, Copy)]
struct CodecReference {
    reference_crf: f64,
    reference_bits_per_pixel: f64,
}

fn codec_reference(codec: &str) -> CodecReference {
    match codec.to_lowercase().as_str() {
        "libx265" | "h265" | "hevc" => CodecReference {
            reference_crf: 28.0,
            reference_bits_per_pixel: DEFAULT_H265_BITS_PER_PIXEL,
        },
        "libvpx-vp9" | "vp9" => CodecReference {
            reference_crf: 31.0,
            reference_bits_per_pixel: DEFAULT_VP9_BITS_PER_PIXEL,
        },
        "libaom-av1" | "av1" | "libsvtav1" => CodecReference {
            reference_crf: 32.0,
            reference_bits_per_pixel: DEFAULT_AV1_BITS_PER_PIXEL,
        },
        "prores" | "prores_ks" => CodecReference {
            reference_crf: 9.0,
            reference_bits_per_pixel: DEFAULT_PRORES_BITS_PER_PIXEL,
        },
        "h264_videotoolbox" | "h264_nvenc" => CodecReference {
            reference_crf: 23.0,
            reference_bits_per_pixel: 0.045,
        },
        _ => CodecReference {
            reference_crf: 23.0,
            reference_bits_per_pixel: DEFAULT_H264_BITS_PER_PIXEL,
        },
    }
}

fn reference_bitrate_from_quality(codec: &str, crf: f64, pixel_rate: f64) -> f64 {
    let reference = codec_reference(codec);
    let quality_factor = 2f64.powf((reference.reference_crf - crf) / 6.0);
    (reference.reference_bits_per_pixel * quality_factor * pixel_rate) / 1000.0
}

#[command]
pub async fn estimate_output(
    config: ConversionConfig,
    metadata: Option<ProbeMetadata>,
) -> Result<OutputEstimate, ConversionError> {
    let metadata_ref = metadata.as_ref();
    let audio_only = is_audio_only_container(&config.container);

    let target_dimensions = determine_target_dimensions(&config, metadata_ref)?;
    let target_fps = determine_target_fps(&config, metadata_ref)?;

    let video_kbps = if audio_only {
        0.0
    } else if config.video_bitrate_mode == "bitrate" {
        parse_config_bitrate(&config.video_bitrate, "video bitrate")?
    } else {
        estimate_quality_video_bitrate(&config, metadata_ref, target_dimensions, target_fps)
    };

    let audio_kbps = estimate_audio_bitrate_kbps(&config, metadata_ref, audio_only)?;

    let total_payload_kbps = video_kbps + audio_kbps;
    let total_kbps_with_overhead =
        total_payload_kbps * container_overhead_factor(&config.container);

    let size_mb = parse_duration_to_seconds(metadata_ref.and_then(|m| m.duration.as_ref()))
        .map(|seconds| (total_kbps_with_overhead * seconds) / 8.0 / 1000.0);

    Ok(OutputEstimate {
        video_kbps: video_kbps.round() as u32,
        audio_kbps: audio_kbps.round() as u32,
        total_kbps: total_kbps_with_overhead.round() as u32,
        size_mb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::async_runtime;

    fn sample_config(container: &str) -> ConversionConfig {
        ConversionConfig {
            container: container.into(),
            video_codec: "libx264".into(),
            video_bitrate_mode: "crf".into(),
            video_bitrate: "5000".into(),
            audio_codec: "aac".into(),
            audio_bitrate: "128".into(),
            audio_channels: "original".into(),
            selected_audio_tracks: vec![],
            resolution: "original".into(),
            custom_width: None,
            custom_height: None,
            scaling_algorithm: "bicubic".into(),
            fps: "original".into(),
            crf: 23,
            quality: 50,
            preset: "medium".into(),
        }
    }

    fn sample_metadata() -> ProbeMetadata {
        ProbeMetadata {
            duration: Some("60".into()),
            bitrate: Some("4128000".into()),
            video_codec: Some("h264".into()),
            audio_codec: Some("aac".into()),
            resolution: Some("1920x1080".into()),
            frame_rate: Some(30.0),
            width: Some(1920),
            height: Some(1080),
            video_bitrate_kbps: Some(4000.0),
            audio_tracks: vec![AudioTrack {
                index: 0,
                codec: "aac".into(),
                channels: "2".into(),
                language: None,
                label: None,
                bitrate_kbps: Some(128.0),
            }],
        }
    }

    #[test]
    fn test_estimate_output_standard_video() {
        let config = sample_config("mp4");
        let metadata = sample_metadata();

        let estimate = async_runtime::block_on(async {
            estimate_output(config, Some(metadata)).await.unwrap()
        });

        assert_eq!(estimate.audio_kbps, 128);
        assert_eq!(estimate.video_kbps, 4666);
        assert_eq!(estimate.total_kbps, 4832);
        let size = estimate.size_mb.expect("size should exist");
        assert!((size - 36.2).abs() < 0.2);
    }

    #[test]
    fn test_estimate_output_without_audio_stream() {
        let config = sample_config("mp4");
        let mut metadata = sample_metadata();
        metadata.audio_codec = None;
        metadata.audio_tracks.clear();

        let estimate = async_runtime::block_on(async {
            estimate_output(config, Some(metadata)).await.unwrap()
        });

        assert_eq!(estimate.audio_kbps, 0);
        assert!(estimate.video_kbps > 0);
    }

    #[test]
    fn test_estimate_output_audio_only_container() {
        let mut config = sample_config("mp3");
        config.audio_codec = "mp3".into();
        let metadata = sample_metadata();

        let estimate = async_runtime::block_on(async {
            estimate_output(config, Some(metadata)).await.unwrap()
        });

        assert_eq!(estimate.video_kbps, 0);
        assert_eq!(estimate.audio_kbps, 128);
        assert_eq!(estimate.total_kbps, 129);
    }

    #[test]
    fn test_estimate_output_without_metadata_uses_reference_curve() {
        let mut config = sample_config("mp4");
        config.resolution = "720p".into();
        config.fps = "30".into();

        let estimate =
            async_runtime::block_on(async { estimate_output(config, None).await.unwrap() });

        assert_eq!(estimate.video_kbps, 2074);
    }

    #[test]
    fn test_audio_copy_prefers_probe_bitrate() {
        let mut config = sample_config("mp4");
        config.audio_codec = "copy".into();
        config.audio_bitrate = "256".into();
        config.selected_audio_tracks = vec![0];

        let metadata = sample_metadata();
        let estimate = async_runtime::block_on(async {
            estimate_output(config, Some(metadata)).await.unwrap()
        });

        assert_eq!(estimate.audio_kbps, 128);
    }

    #[test]
    fn test_estimate_output_crf_adjustment_tracks_source_quality() {
        let metadata = sample_metadata();
        let mut baseline_config = sample_config("mp4");
        baseline_config.crf = 23;

        let baseline = async_runtime::block_on(async {
            estimate_output(baseline_config.clone(), Some(metadata.clone()))
                .await
                .unwrap()
        });

        let mut lower_quality = baseline_config.clone();
        lower_quality.crf = 30;

        let mut higher_quality = baseline_config;
        higher_quality.crf = 18;

        let low_estimate = async_runtime::block_on(async {
            estimate_output(lower_quality, Some(metadata.clone()))
                .await
                .unwrap()
        });
        let high_estimate = async_runtime::block_on(async {
            estimate_output(higher_quality, Some(metadata))
                .await
                .unwrap()
        });

        assert!(low_estimate.video_kbps < baseline.video_kbps);
        assert!(high_estimate.video_kbps > baseline.video_kbps);
    }
}
