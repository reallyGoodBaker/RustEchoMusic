<script lang="ts">
    import TextField from './TextField.svelte'

    type SearchFieldProps = {
        value?: string
        placeholder?: string
        class?: string
        onchange?: (value: string, event: Event) => void
        oninput?: (value: string, event: Event) => void
        [key: string]: unknown
    }

    let {
        value = $bindable(''),
        placeholder = '搜索...',
        class: className = '',
        onchange,
        oninput,
        ...props
    }: SearchFieldProps = $props()
</script>

<TextField
    bind:value
    variant="outlined"
    type="search"
    icon="search"
    clearable
    {placeholder}
    class={className}
    {onchange}
    {oninput}
    {...props}
/>