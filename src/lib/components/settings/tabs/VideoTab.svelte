<script lang="ts">
	import type { ConversionConfig } from '$lib/types';
	import Button from '$lib/components/ui/Button.svelte';
	import ListItem from '$lib/components/ui/ListItem.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import Slider from '$lib/components/ui/Slider.svelte';
	import { platform } from '@tauri-apps/plugin-os';

	const RESOLUTIONS = ['original', '1080p', '720p', '480p', 'custom'] as const;
	const ALL_VIDEO_CODECS = [
		{ id: 'libx264', label: 'H.264 / AVC' },
		{ id: 'libx265', label: 'H.265 / HEVC' },
		{ id: 'vp9', label: 'VP9 / Web' },
		{ id: 'prores', label: 'Apple ProRes' },
		{ id: 'libsvtav1', label: 'AV1 / SVT' },
		{ id: 'h264_videotoolbox', label: 'H.264 (Apple Silicon)' },
		{ id: 'h264_nvenc', label: 'H.264 (NVIDIA)' }
	] as const;

	const currentPlatform = platform();

	const availableCodecs = ALL_VIDEO_CODECS.filter((codec) => {
		if (codec.id === 'h264_videotoolbox') return currentPlatform === 'macos';
		if (codec.id === 'h264_nvenc') return currentPlatform === 'windows' || currentPlatform === 'linux';
		return true;
	});

	const PRESETS = [
		{ id: 'ultrafast', label: 'Ultrafast', desc: 'Largest file, fastest conversion' },
		{ id: 'superfast', label: 'Superfast', desc: 'Very large file' },
		{ id: 'veryfast', label: 'Veryfast', desc: 'Large file' },
		{ id: 'faster', label: 'Faster', desc: 'Moderately large file' },
		{ id: 'fast', label: 'Fast', desc: 'Slightly larger file' },
		{ id: 'medium', label: 'Medium', desc: 'Balanced' },
		{ id: 'slow', label: 'Slow', desc: 'Better compression' },
		{ id: 'slower', label: 'Slower', desc: 'High compression' },
		{ id: 'veryslow', label: 'Veryslow', desc: 'Smallest file, slowest conversion' }
	] as const;

	const SCALING_ALGOS = [
		{ id: 'bicubic', label: 'Bicubic' },
		{ id: 'lanczos', label: 'Lanczos' },
		{ id: 'bilinear', label: 'Bilinear' },
		{ id: 'nearest', label: 'Nearest' }
	] as const;

	const FPS_OPTIONS = [
		{ id: 'original', label: 'Same as source' },
		{ id: '24', label: '24 fps' },
		{ id: '30', label: '30 fps' },
		{ id: '60', label: '60 fps' }
	] as const;

	const isHardwareEncoder = $derived(
		config.videoCodec === 'h264_videotoolbox' || config.videoCodec === 'h264_nvenc'
	);

	let {
		config,
		disabled = false,
		onUpdate
	}: {
		config: ConversionConfig;
		disabled?: boolean;
		onUpdate: (config: Partial<ConversionConfig>) => void;
	} = $props();
</script>

<div class="space-y-4">
	<div class="space-y-3">
		<Label variant="section">Resolution & Framerate</Label>
		<div class="mb-2 grid grid-cols-2 gap-2">
			{#each RESOLUTIONS as res (res)}
				<Button
					variant={config.resolution === res ? 'selected' : 'outline'}
					onclick={() => onUpdate({ resolution: res })}
					{disabled}
					class="w-full"
				>
					{res}
				</Button>
			{/each}
		</div>

		{#if config.resolution === 'custom'}
			<div class="mb-2 grid grid-cols-2 gap-2 pt-1">
				<div class="flex flex-col gap-1">
					<Label for="width">Width</Label>
					<Input
						id="width"
						type="text"
						inputmode="numeric"
						placeholder="1920"
						value={config.customWidth}
						oninput={(e) => {
							const value = e.currentTarget.value.replace(/[^0-9]/g, '');
							onUpdate({ customWidth: value });
						}}
						{disabled}
					/>
				</div>
				<div class="flex flex-col gap-1">
					<Label for="height">Height</Label>
					<Input
						id="height"
						type="text"
						inputmode="numeric"
						placeholder="1080"
						value={config.customHeight}
						oninput={(e) => {
							const value = e.currentTarget.value.replace(/[^0-9]/g, '');
							onUpdate({ customHeight: value });
						}}
						{disabled}
					/>
				</div>
			</div>
		{/if}

		<div class="space-y-3 pt-2">
			<Label variant="section">Scaling Algorithm</Label>
			<div class="grid grid-cols-2 gap-2">
				{#each SCALING_ALGOS as algo (algo.id)}
					<Button
						variant={config.scalingAlgorithm === algo.id ? 'selected' : 'outline'}
						onclick={() => onUpdate({ scalingAlgorithm: algo.id })}
						disabled={disabled || config.resolution === 'original'}
						class="w-full"
					>
						{algo.label}
					</Button>
				{/each}
			</div>
		</div>

		<div class="space-y-3 pt-2">
			<Label variant="section">Framerate</Label>
			<div class="grid grid-cols-2 gap-2">
				{#each FPS_OPTIONS as opt (opt.id)}
					<Button
						variant={config.fps === opt.id ? 'selected' : 'outline'}
						onclick={() => onUpdate({ fps: opt.id })}
						{disabled}
						class="w-full"
					>
						{opt.label}
					</Button>
				{/each}
			</div>
		</div>
	</div>

	<div class="space-y-3 pt-2">
		<Label variant="section">Video Encoder</Label>
		<div class="grid grid-cols-1 gap-1.5">
			{#each availableCodecs as codec (codec.id)}
				<ListItem
					selected={config.videoCodec === codec.id}
					onclick={() => onUpdate({ videoCodec: codec.id })}
					{disabled}
				>
					<span>{codec.id}</span>
					<span class="text-[9px] opacity-50">{codec.label}</span>
				</ListItem>
			{/each}
		</div>
	</div>

	<div class="space-y-3 pt-2">
		<Label variant="section">Encoding Speed</Label>
		<div class="grid grid-cols-1 gap-1.5">
			{#each PRESETS as preset (preset.id)}
				<ListItem
					selected={config.preset === preset.id}
					onclick={() => onUpdate({ preset: preset.id })}
					{disabled}
				>
					<span>{preset.label}</span>
					<span class="text-[9px] opacity-50">{preset.desc}</span>
				</ListItem>
			{/each}
		</div>
	</div>

	<div class="space-y-3 pt-2">
		<Label variant="section">Quality Control</Label>
		<div class="grid grid-cols-2 gap-2">
			<Button
				variant={config.videoBitrateMode === 'crf' ? 'selected' : 'outline'}
				onclick={() => onUpdate({ videoBitrateMode: 'crf' })}
				{disabled}
				class="w-full"
			>
				Constant Quality
			</Button>
			<Button
				variant={config.videoBitrateMode === 'bitrate' ? 'selected' : 'outline'}
				onclick={() => onUpdate({ videoBitrateMode: 'bitrate' })}
				{disabled}
				class="w-full"
			>
				Target Bitrate
			</Button>
		</div>
	</div>

	{#if config.videoBitrateMode === 'crf'}
		<div class="space-y-2 pt-2">
			<div class="flex items-end justify-between">
				<Label for="quality-factor">
					{#if isHardwareEncoder}
						Encoding Quality
					{:else}
						Quality Factor
					{/if}
				</Label>
				<div
					class="rounded border border-ds-blue-600 bg-ds-blue-900/20 px-1.5 text-[10px] font-medium text-ds-blue-600"
				>
					{#if isHardwareEncoder}
						Q {config.quality}
					{:else}
						CRF {config.crf}
					{/if}
				</div>
			</div>
			<div class="py-2">
				{#if isHardwareEncoder}
					<Slider
						id="quality-factor"
						min={1}
						max={100}
						step={1}
						value={config.quality}
						oninput={(e) => onUpdate({ quality: parseInt(e.currentTarget.value) })}
						{disabled}
					/>
				{:else}
					<Slider
						id="quality-factor"
						min={0}
						max={51}
						value={config.crf}
						oninput={(e) => onUpdate({ crf: parseInt(e.currentTarget.value) })}
						{disabled}
					/>
				{/if}
			</div>
			<div class="flex justify-between text-[9px] uppercase text-gray-alpha-600">
				{#if isHardwareEncoder}
					<span>Low Quality</span>
					<span>Best Quality</span>
				{:else}
					<span>Lossless</span>
					<span>Smallest</span>
				{/if}
			</div>
		</div>
	{:else}
		<div class="space-y-2 pt-1">
			<div class="flex items-end justify-between">
				<Label for="video-bitrate">Target Bitrate (kbps)</Label>
			</div>
			<div class="flex items-center gap-2">
				<Input
					id="video-bitrate"
					type="text"
					inputmode="numeric"
					value={config.videoBitrate}
					oninput={(e) => {
						const value = e.currentTarget.value.replace(/[^0-9]/g, '');
						onUpdate({ videoBitrate: value });
					}}
					{disabled}
				/>
			</div>
		</div>
	{/if}
</div>
