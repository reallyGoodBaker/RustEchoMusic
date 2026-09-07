<script lang="ts">
    import type { Snippet } from 'svelte'

    type SettingsSectionProps = {
        title: string
        children?: Snippet
        class?: string
        contentClass?: string
    }

    let {
        title,
        children,
        class: className = '',
        contentClass = 'mt-4 grid gap-4',
    }: SettingsSectionProps = $props()
</script>

<section
    class={`rounded-3xl bg-[rgb(var(--mdui-color-surface-container))] p-5 ${className}`}
>
    <h2
        class="text-lg font-semibold text-[rgb(var(--mdui-color-on-surface))]"
    >
        {title}
    </h2>

    {#if children}
        <div class={contentClass}>
            {@render children()}
        </div>
    {/if}
</section>