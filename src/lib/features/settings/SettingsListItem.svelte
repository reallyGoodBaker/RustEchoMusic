<script lang="ts">
    import type { Snippet } from 'svelte'

    type SettingsListItemProps = {
        title: string
        description?: string
        children?: Snippet
        class?: string
    }

    let {
        title,
        description,
        children,
        class: className = '',
    }: SettingsListItemProps = $props()
</script>

<div
    class={`flex items-center justify-between gap-4 rounded-2xl bg-[rgb(var(--mdui-color-surface-container-highest))] px-4 py-3 ${className}`}
>
    <div class="min-w-0">
        <div class="truncate text-sm font-medium">
            {title}
        </div>

        {#if description}
            <div
                class="mt-1 text-xs text-[rgb(var(--mdui-color-on-surface-variant))]"
            >
                {description}
            </div>
        {/if}
    </div>

    {#if children}
        <div class="shrink-0">
            {@render children()}
        </div>
    {/if}
</div>