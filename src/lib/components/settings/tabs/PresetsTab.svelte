<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Trash2 } from 'lucide-svelte';
	import { cn } from '$lib/utils/cn';
	import type { ConversionConfig, PresetDefinition } from '$lib/types';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import ListItem from '$lib/components/ui/ListItem.svelte';
	import Label from '$lib/components/ui/Label.svelte';

	let {
		config,
		presets = [],
		disabled = false,
		onApplyPreset,
		onSavePreset,
		onDeletePreset
	}: {
		config: ConversionConfig;
		presets?: PresetDefinition[];
		disabled?: boolean;
		onApplyPreset?: (preset: PresetDefinition) => void;
		onSavePreset?: (name: string) => Promise<boolean | void> | boolean | void;
		onDeletePreset?: (id: string) => Promise<boolean | void> | boolean | void;
	} = $props();

	let newPresetName = $state('');
	type NoticeTone = 'success' | 'error';
	let notice = $state<{ text: string; tone: NoticeTone } | null>(null);
	let noticeTimeout: ReturnType<typeof setTimeout> | null = null;

	onDestroy(() => {
		if (noticeTimeout) clearTimeout(noticeTimeout);
	});

	function configsMatch(a: ConversionConfig, b: ConversionConfig) {
		return (
			a.container === b.container &&
			a.videoCodec === b.videoCodec &&
			a.audioCodec === b.audioCodec &&
			a.resolution === b.resolution &&
			a.crf === b.crf &&
			a.preset === b.preset
		);
	}

	function showNotice(text: string, tone: NoticeTone = 'success') {
		notice = { text, tone };
		if (noticeTimeout) clearTimeout(noticeTimeout);
		noticeTimeout = setTimeout(() => (notice = null), 2400);
	}

	async function savePreset() {
		if (!onSavePreset || disabled) return;
		if (!newPresetName.trim()) {
			showNotice('Name required', 'error');
			return;
		}

		const result = await onSavePreset(newPresetName.trim());
		if (result === false) {
			showNotice('Preset not saved', 'error');
			return;
		}

		newPresetName = '';
		showNotice('Preset saved');
	}

	function applyPreset(preset: PresetDefinition) {
		if (disabled) return;
		onApplyPreset?.(preset);
		showNotice(`Applied ${preset.name}`);
	}

	async function removePreset(preset: PresetDefinition) {
		if (!onDeletePreset || preset.builtIn) return;
		const result = await onDeletePreset(preset.id);
		if (result === false) {
			showNotice('Unable to delete', 'error');
			return;
		}

		showNotice('Preset removed');
	}
</script>

<div class="space-y-3">
	<div class="relative w-full">
		<Label variant="section">Preset Library</Label>
		{#if notice}
			<span
				class={cn(
					'absolute top-0 right-0 text-[9px] tracking-wide uppercase',
					notice.tone === 'error' ? 'text-ds-red-700' : 'text-ds-blue-600'
				)}
			>
				{notice.text}
			</span>
		{/if}
	</div>

	<div class="flex gap-2">
		<div class="flex-1">
			<Input
				type="text"
				value={newPresetName}
				oninput={(e) => (newPresetName = e.currentTarget.value)}
				placeholder="Preset Label"
				{disabled}
			/>
		</div>
		<Button onclick={savePreset} disabled={disabled || !newPresetName.trim()} variant="outline">
			Save
		</Button>
	</div>

	<div class="space-y-1.5">
		{#each presets as preset (preset.id)}
			<ListItem
				selected={configsMatch(config, preset.config)}
				onclick={() => applyPreset(preset)}
				onkeydown={(event) => {
					if (event.key === 'Enter' || event.key === ' ') {
						event.preventDefault();
						applyPreset(preset);
					}
				}}
				class="pr-1"
			>
				<span class="truncate">{preset.name}</span>
				<div class="flex items-center gap-2">
					<span class="pr-2 text-[9px] font-medium opacity-50">
						{configsMatch(config, preset.config) ? 'APPLIED' : ''}
					</span>
					{#if !preset.builtIn}
						<Button
							variant="destructive"
							size="none"
							class="size-5 shrink-0 opacity-50 hover:opacity-100"
							title="Delete preset"
							onclick={(event) => {
								event.stopPropagation();
								removePreset(preset);
							}}
							{disabled}
						>
							<Trash2 size={12} />
						</Button>
					{/if}
				</div>
			</ListItem>
		{/each}
	</div>
</div>
