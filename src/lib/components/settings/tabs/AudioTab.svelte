<script lang="ts">
    import type { ConversionConfig, SourceMetadata } from "$lib/types";

    const AUDIO_CODECS = [
        { id: "aac", label: "AAC / Stereo" },
        { id: "ac3", label: "Dolby Digital" },
        { id: "libopus", label: "Opus" },
        { id: "mp3", label: "MP3" },
    ] as const;

    const CHANNELS = [
        { id: "original", label: "Original" },
        { id: "stereo", label: "Stereo (2.0)" },
        { id: "mono", label: "Mono (1.0)" },
    ] as const;

    let {
        config,
        disabled = false,
        onUpdate,
        metadata,
    }: {
        config: ConversionConfig;
        disabled?: boolean;
        onUpdate: (config: Partial<ConversionConfig>) => void;
        metadata?: SourceMetadata;
    } = $props();

    function toggleTrack(index: number) {
        if (disabled) return;
        const current = config.selectedAudioTracks || [];
        if (current.includes(index)) {
            onUpdate({
                selectedAudioTracks: current.filter((i) => i !== index),
            });
        } else {
            onUpdate({ selectedAudioTracks: [...current, index] });
        }
    }
</script>

<div class="space-y-4">
    <div class="space-y-3">
        <span
            class="text-[10px] text-gray-alpha-600 uppercase tracking-widest block border-b border-gray-alpha-100 pb-1"
        >
            Channels & Bitrate
        </span>
        <div class="space-y-3">
            <div class="grid grid-cols-3 gap-2">
                {#each CHANNELS as ch}
                    <button
                        onclick={() =>
                            onUpdate({ audioChannels: ch.id as any })}
                        {disabled}
                        class="text-[11px] py-1.5 px-2 border rounded transition-all text-center uppercase
                        {config.audioChannels === ch.id
                            ? 'bg-ds-blue-900/20 text-ds-blue-600 border-ds-blue-600'
                            : 'bg-transparent text-gray-alpha-600 border-gray-alpha-200 hover:bg-gray-alpha-100 hover:text-foreground'}"
                    >
                        {ch.label}
                    </button>
                {/each}
            </div>

            <div class="space-y-2 pt-1">
                <label
                    for="audio-bitrate"
                    class="text-[10px] text-gray-alpha-600 uppercase tracking-widest whitespace-nowrap"
                    >Bitrate (kbps)</label
                >
                <input
                    id="audio-bitrate"
                    type="number"
                    value={config.audioBitrate}
                    oninput={(e) =>
                        onUpdate({ audioBitrate: e.currentTarget.value })}
                    class="w-full text-[11px] font-mono uppercase tracking-wide px-3 py-1.5 border border-gray-alpha-200 rounded bg-transparent focus:outline-none focus:border-ds-blue-600! transition-all [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                    {disabled}
                />
            </div>
        </div>
    </div>
    <div class="space-y-3">
        <span
            class="text-[10px] text-gray-alpha-600 uppercase tracking-widest block border-b border-gray-alpha-100 pb-1"
        >
            Audio Codec
        </span>
        <div class="grid grid-cols-1">
            {#each AUDIO_CODECS as codec}
                {@const isMp3Container = config.container === "mp3"}
                {@const isAllowed = !isMp3Container || codec.id === "mp3"}
                <button
                    onclick={() => onUpdate({ audioCodec: codec.id })}
                    disabled={disabled || !isAllowed}
                    class="text-[11px] py-1.5 px-3 border-l-2 text-left transition-all uppercase flex justify-between
                    {config.audioCodec === codec.id
                        ? 'border-l-ds-blue-600 bg-gray-alpha-100 text-foreground pl-3'
                        : 'border-l-transparent text-gray-alpha-600 hover:text-foreground pl-2'}
                    {!isAllowed ? 'opacity-50 cursor-not-allowed' : ''}"
                >
                    <span>{codec.id}</span>
                    <span class="opacity-50 text-[9px]">
                        {isMp3Container && codec.id !== "mp3"
                            ? "Incompatible with MP3"
                            : codec.label}
                    </span>
                </button>
            {/each}
        </div>
    </div>

    {#if metadata?.audioTracks && metadata.audioTracks.length > 0}
        <div class="space-y-3 pt-1">
            <span
                class="text-[10px] text-gray-alpha-600 uppercase tracking-widest block border-b border-gray-alpha-100 pb-1"
            >
                Source Tracks
            </span>
            <div class="space-y-1.5">
                {#each metadata.audioTracks as track}
                    <button
                        onclick={() => toggleTrack(track.index)}
                        {disabled}
                        class="w-full text-[11px] py-1.5 px-3 border rounded text-left transition-all flex items-center justify-between
                        {(config.selectedAudioTracks || []).includes(
                            track.index,
                        )
                            ? 'bg-ds-blue-900/10 border-ds-blue-600 text-foreground'
                            : 'bg-transparent border-gray-alpha-200 text-gray-alpha-600 hover:text-foreground hover:bg-gray-alpha-100'}"
                    >
                        <div class="flex flex-col items-start">
                            <span class="uppercase font-medium"
                                >Stream #{track.index}</span
                            >
                            <span class="text-[9px] opacity-70">
                                {track.codec} • {track.channels}ch
                                {#if track.language}• {track.language}{/if}
                            </span>
                        </div>
                        {#if (config.selectedAudioTracks || []).includes(track.index)}
                            <div
                                class="w-2 h-2 rounded-full bg-ds-blue-600"
                            ></div>
                        {/if}
                    </button>
                {/each}
            </div>
        </div>
    {/if}
</div>
