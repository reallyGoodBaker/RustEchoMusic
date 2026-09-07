<script lang="ts">

    type PageHeadingProps = {
        eyebrow: string
        title: string
        class?: string
    }

    let {
        eyebrow,
        title,
        class: className = '',
    }: PageHeadingProps = $props()
</script>

<div class="mx-2">
    <div>
        <p
            class="text-xs font-semibold uppercase tracking-wider text-[rgb(var(--mdui-color-primary))]"
        >
            {eyebrow}
        </p>

        <h1
            class="mt-1 text-3xl font-bold tracking-tight text-[rgb(var(--mdui-color-on-surface))]"
        >
            {title}
        </h1>
    </div>
</div>
