<script lang="ts">
    import { v4 as uuidv4 } from "uuid";
    import { open } from "@tauri-apps/plugin-dialog";
    import { stat } from "@tauri-apps/plugin-fs";

    import Titlebar from "$lib/components/Titlebar.svelte";
    import FileList from "$lib/components/FileList.svelte";
    import SettingsPanel from "$lib/components/SettingsPanel.svelte";
    import EmptySelection from "$lib/components/EmptySelection.svelte";
    import {
        type FileItem,
        FileStatus,
        type ConversionConfig,
    } from "$lib/types";
    import {
        startConversion as startConversionService,
        setupConversionListeners,
    } from "$lib/services/conversion";

    const DEFAULT_CONFIG: ConversionConfig = {
        container: "mp4",
        videoCodec: "libx264",
        audioCodec: "aac",
        resolution: "original",
        crf: 23,
        preset: "medium",
    };

    let files = $state<FileItem[]>([]);
    let selectedFileId = $state<string | null>(null);
    let isProcessing = $state(false);

    let selectedFile = $derived(files.find((f) => f.id === selectedFileId));
    let totalSize = $derived(files.reduce((acc, curr) => acc + curr.size, 0));

    $effect(() => {
        const unlistenPromise = setupConversionListeners(
            (payload) => {
                files = files.map((f) =>
                    f.id === payload.id
                        ? { ...f, progress: payload.progress }
                        : f,
                );
            },
            (payload) => {
                files = files.map((f) =>
                    f.id === payload.id
                        ? { ...f, status: FileStatus.COMPLETED, progress: 100 }
                        : f,
                );
                checkAllDone();
            },
            (payload) => {
                files = files.map((f) =>
                    f.id === payload.id
                        ? { ...f, status: FileStatus.ERROR }
                        : f,
                );
                checkAllDone();
            },
        );

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    });

    function checkAllDone() {
        if (
            files.every(
                (f) =>
                    f.status === FileStatus.COMPLETED ||
                    f.status === FileStatus.ERROR ||
                    f.status === FileStatus.IDLE,
            )
        ) {
            isProcessing = false;
        }
    }

    async function handleAddFile() {
        const selected = await open({
            multiple: true,
            filters: [
                {
                    name: "Videos",
                    extensions: ["mp4", "mov", "mkv", "avi", "webm"],
                },
            ],
        });

        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected];

            const newFiles: FileItem[] = [];

            for (const pathStr of paths) {
                const name = pathStr.split(/[/\\]/).pop() || "unknown";
                let size = 0;
                try {
                    const metadata = await stat(pathStr);
                    size = metadata.size;
                } catch (e) {
                    console.error("Failed to stat file:", pathStr, e);
                }

                newFiles.push({
                    id: uuidv4(),
                    name: name,
                    size: size,
                    status: FileStatus.IDLE,
                    progress: 0,
                    originalFormat: name.split(".").pop() || "unknown",
                    config: { ...DEFAULT_CONFIG },
                    path: pathStr,
                });
            }

            files = [...files, ...newFiles];
            if (!selectedFileId && newFiles.length > 0) {
                selectedFileId = newFiles[0].id;
            }
        }
    }

    function handleRemoveFile(id: string) {
        files = files.filter((f) => f.id !== id);
        if (selectedFileId === id) selectedFileId = null;
    }

    function updateSelectedConfig(newConfig: Partial<ConversionConfig>) {
        if (selectedFileId) {
            files = files.map((f) =>
                f.id === selectedFileId
                    ? { ...f, config: { ...f.config, ...newConfig } }
                    : f,
            );
        }
    }

    async function startConversion() {
        const pendingFiles = files.filter((f) => f.status === FileStatus.IDLE);
        if (pendingFiles.length === 0) return;

        isProcessing = true;

        files = files.map((f) =>
            f.status === FileStatus.IDLE
                ? { ...f, status: FileStatus.CONVERTING, progress: 0 }
                : f,
        );

        for (const file of pendingFiles) {
            await startConversionService(file.id, file.path, file.config);
        }
    }
</script>

<div
    class="flex flex-col absolute inset-0 text-foreground font-sans overflow-hidden selection:bg-ds-blue-900 selection:text-white"
>
    <Titlebar
        {totalSize}
        fileCount={files.length}
        {isProcessing}
        onAddFile={handleAddFile}
        onStartConversion={startConversion}
    />
    <div class="flex-1 p-4 overflow-hidden">
        <div class="grid grid-cols-12 gap-4 h-full">
            <FileList
                {files}
                {selectedFileId}
                onSelect={(id) => (selectedFileId = id)}
                onRemove={handleRemoveFile}
            />

            <div
                class="col-span-12 lg:col-span-4 border border-ds-gray-100 rounded-lg overflow-hidden flex flex-col"
            >
                {#if selectedFile}
                    <SettingsPanel
                        config={selectedFile.config}
                        onUpdate={updateSelectedConfig}
                        disabled={selectedFile.status ===
                            FileStatus.CONVERTING ||
                            selectedFile.status === FileStatus.COMPLETED}
                    />
                {:else}
                    <EmptySelection />
                {/if}
            </div>
        </div>
    </div>
</div>
