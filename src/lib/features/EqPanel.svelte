<script lang="ts">
    import { onMount } from 'svelte'
    import { equalizer, BAND_LABELS, EQ_PRESETS } from '$lib/state/equalizer.svelte'
    import Select from '$lib/components/base/Select.svelte'
    import Slider from '$lib/components/base/Slider.svelte'
    import 'mdui/components/switch.js'

    let debounceTimers: (ReturnType<typeof setTimeout> | null)[] = $state(Array(BAND_LABELS.length).fill(null))

    onMount(() => {
        if (!equalizer.loaded) void equalizer.load()
    })

    function handleBandChange(index: number, value: number) {
        if (debounceTimers[index]) clearTimeout(debounceTimers[index])
        debounceTimers[index] = setTimeout(() => {
            void equalizer.setBand(index, value)
        }, 50)
    }

    function handlePresetChange(value: string) {
        const preset = EQ_PRESETS.find((p) => p.name === value)
        if (preset) void equalizer.applyPreset(preset)
    }
</script>

<div
    class="eq-panel flex h-full min-w-70 flex-col gap-3 overflow-hidden rounded-2xl bg-[rgb(var(--mdui-color-surface-container))] p-4 text-[rgb(var(--mdui-color-on-surface))]"
>
    <div class="eq-header flex items-center justify-between">
        <span class="eq-title font-semibold">均衡器</span>
        <mdui-switch
            checked={equalizer.enabled}
            onchange={(e: Event) =>
                equalizer.setEnabled((e.currentTarget as HTMLInputElement).checked)}
        ></mdui-switch>
    </div>
    <Select
        value={equalizer.presetName}
        options={EQ_PRESETS.map((p) => ({ label: p.name, value: p.name }))}
        onchange={handlePresetChange}
    />
    <div class="eq-bands flex flex-1 items-end justify-around gap-1">
        {#each BAND_LABELS as label, i (label)}
            <div class="eq-band flex flex-col items-center gap-1">
                <Slider
                    vertical
                    min={-12}
                    max={12}
                    step={0.1}
                    width={120}
                    height={12}
                    thumbHeight={24}
                    disabled={!equalizer.enabled}
                    bind:value={equalizer.bands[i]}
                    onchange={(v) => handleBandChange(i, v)}
                />
                <span class="text-[0.7rem] text-[rgb(var(--mdui-color-on-surface-variant))]">{label}</span>
            </div>
        {/each}
    </div>
</div>
