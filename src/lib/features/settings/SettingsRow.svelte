<script lang="ts">
    import type { Snippet } from 'svelte'

    type SettingsRowProps = {
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
    }: SettingsRowProps = $props()
</script>

<div class={`flex items-center justify-between gap-4 ${className}`}>
    <div class="min-w-0">
        <div class="text-sm font-medium text-[rgb(var(--mdui-color-on-surface))]">{title}</div>

        {#if description}
            <div
                class="text-xs text-[rgb(var(--mdui-color-on-surface-variant))]"
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