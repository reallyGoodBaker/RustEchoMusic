<script lang="ts">
    import { createVirtualizer } from '@tanstack/svelte-virtual'
    import { untrack } from 'svelte'
    import 'mdui/components/button-icon.js'
    import 'mdui/components/card.js'

    export type MediaGridItem = {
        id: string | number
        title: string
        subtitle?: string
        image?: string | null
        href?: string
        shape?: 'square' | 'circle'
    }

    let {
        items,
        emptyTitle = '暂无内容',
        emptyDescription = '这里还没有可以显示的内容',
        selectedId = null,
        onselect,
        onplay,
    }: {
        items: MediaGridItem[]
        emptyTitle?: string
        emptyDescription?: string
        selectedId?: string | number | null
        onselect?: (item: MediaGridItem) => void
        onplay?: (item: MediaGridItem, event: Event) => void
    } = $props()

    let scrollContainer = $state<HTMLElement | null>(null)
    let containerWidth = $state(0)

    let columns = $derived.by(() => {
        const width = containerWidth || 800
        if (width >= 1280) return 8
        if (width >= 1024) return 7
        if (width >= 768) return 6
        if (width >= 640) return 5
        return 4
    })

    let rowCount = $derived(Math.ceil(items.length / columns))

    let rowHeight = $derived.by(() => {
        if (!containerWidth || !columns) return 250
        const gap = 16
        const colWidth = (containerWidth - gap * (columns - 1)) / columns
        return colWidth + 72
    })

    const options = $derived({
        count: rowCount, 
        getScrollElement: () => scrollContainer,
        estimateSize: () => rowHeight,
    })
    
    const virtualizer = $derived.by(() => createVirtualizer(options))

    $effect(() => {
        const count = rowCount
        const element = scrollContainer
        const size = rowHeight

        untrack(() => {
            $virtualizer.setOptions({
                count,
                getScrollElement: () => element,
                estimateSize: () => size,
                overscan: 3,
            })
            $virtualizer.measure()
        })
    })

    function handleSelect(item: MediaGridItem) {
        onselect?.(item)
    }

    function handleKeydown(item: MediaGridItem, event: KeyboardEvent) {
        if (event.key !== 'Enter' && event.key !== ' ') return
        event.preventDefault()
        handleSelect(item)
    }

    function handlePlay(item: MediaGridItem, event: Event) {
        event.preventDefault()
        event.stopPropagation()
        onplay?.(item, event)
    }

    function handlePlayKeydown(item: MediaGridItem, event: KeyboardEvent) {
        if (event.key !== 'Enter' && event.key !== ' ') return
        handlePlay(item, event)
    }
</script>

{#snippet mediaCardContent(item: MediaGridItem, isSelected: boolean)}
    <div
        class="relative mb-3 aspect-square bg-[rgb(var(--mdui-color-surface-container-highest))] {item.shape === 'circle' ? 'rounded-full' : 'rounded-t-xl'}"
    >
        {#if item.image}
            <img
                src={item.image}
                alt={item.title}
                class="h-full w-full object-cover transition-transform duration-200 group-hover:scale-105"
                loading="lazy"
            />
        {:else}
            <div
                class="flex h-full w-full items-center justify-center text-4xl text-[rgb(var(--mdui-color-on-surface-variant))]"
            >
                {#if item.shape === 'circle'}
                    👤
                {:else}
                    🎵
                {/if}
            </div>
        {/if}

        {#if onplay}
            <div
                class="absolute inset-0 flex items-end justify-end p-2 opacity-0 transition-opacity duration-200 group-hover:opacity-100 group-focus-within:opacity-100"
            >
                <mdui-button-icon
                    variant="filled"
                    icon="play_arrow--rounded"
                    aria-label={`播放 ${item.title}`}
                    role="button"
                    tabindex="0"
                    onclick={(event: Event) => handlePlay(item, event)}
                    onkeydown={(event: KeyboardEvent) => handlePlayKeydown(item, event)}
                ></mdui-button-icon>
            </div>
        {/if}
    </div>

    <div class="min-w-0 px-2 pb-2">
        <h2
            class={[
                'truncate text-sm font-semibold',
                isSelected ? 'text-[rgb(var(--mdui-color-on-secondary-container))]' : 'text-[rgb(var(--mdui-color-on-surface))]',
            ]}
        >
            {item.title}
        </h2>

        {#if item.subtitle}
            <p
                class={[
                    'mt-0 truncate text-xs',
                    isSelected ? 'text-[rgb(var(--mdui-color-on-secondary-container))]' : 'text-[rgb(var(--mdui-color-on-surface-variant))]',
                ]}
            >
                {item.subtitle}
            </p>
        {/if}
    </div>
{/snippet}

{#snippet mediaCard(item: MediaGridItem)}
    {@const isSelected = selectedId === item.id}
    {#if onselect}
        <mdui-card
            class={[
                'group cursor-pointer outline-none transition-colors hover:bg-(--mdui-color-secondary-container)/80',
                isSelected && 'bg-[rgb(var(--mdui-color-secondary-container))]',
            ]}
            aria-label={item.title}
            aria-pressed={isSelected}
            onclick={() => handleSelect(item)}
            onkeydown={(event: KeyboardEvent) => handleKeydown(item, event)}
            role="button"
            tabindex="0"
        >
            {@render mediaCardContent(item, isSelected)}
        </mdui-card>
    {:else}
        <mdui-card
            href={item.href ?? undefined}
            class={[
                'group outline-none transition-colors bg-(--mdui-color-secondary-container)/80',
                isSelected && 'bg-[rgb(var(--mdui-color-secondary-container))]',
            ]}
            aria-label={item.title}
        >
            {@render mediaCardContent(item, isSelected)}
        </mdui-card>
    {/if}
{/snippet}

{#if items.length === 0}
    <div
        class="flex min-h-60 flex-col items-center justify-center rounded-3xl border border-dashed border-[rgb(var(--mdui-color-outline-variant))] text-center"
    >
        <div class="text-5xl">🎧</div>
        <div class="mt-3 text-base font-medium">{emptyTitle}</div>
        <div class="mt-1 text-sm text-[rgb(var(--mdui-color-on-surface-variant))]">
            {emptyDescription}
        </div>
    </div>
{:else}
    <section 
        bind:this={scrollContainer} 
        bind:clientWidth={containerWidth}
        class="h-full w-full overflow-y-scroll overflow-x-hidden"
    >
        <div style="height: {$virtualizer.getTotalSize()}px; position: relative; width: 100%;">
            {#each $virtualizer.getVirtualItems() as row (row.index)}
                {@const startIndex = row.index * columns}
                {@const rowItems = items.slice(startIndex, startIndex + columns)}
                
                <div
                    style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({row.start}px);"
                    class="grid gap-4"
                    style:grid-template-columns="repeat({columns}, minmax(0, 1fr))"
                    data-index={row.index}
                >
                    {#each rowItems as item (item.id)}
                        {@render mediaCard(item)}
                    {/each}
                </div>
            {/each}
        </div>
    </section>
{/if}