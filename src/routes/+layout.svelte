<script>
    let { children } = $props();
    import "./layout.css";
</script>

<div
    class="flex flex-col h-screen overflow-hidden bg-black rounded-xl border border-ds-gray-800/20 -mt-6"
>
    <div class="flex-1 relative">
        {@render children()}
    </div>
</div>
