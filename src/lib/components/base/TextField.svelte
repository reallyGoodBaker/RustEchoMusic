<script lang="ts">
    import 'mdui/components/text-field.js'

    type TextFieldProps = {
        value?: string
        variant?: 'filled' | 'outlined'
        type?: string
        icon?: string
        placeholder?: string
        clearable?: boolean
        class?: string
        oninput?: (value: string, event: Event) => void
        onchange?: (value: string, event: Event) => void
        [key: string]: unknown
    }

    let {
        value = $bindable(''),
        variant = 'outlined',
        type = 'text',
        icon,
        placeholder,
        clearable = false,
        class: className = '',
        oninput,
        onchange,
        ...props
    }: TextFieldProps = $props()

    function handleInput(event: Event) {
        const target = event.currentTarget as HTMLElement & { value: string }

        value = target.value
        oninput?.(value, event)
    }

    function handleChange(event: Event) {
        const target = event.currentTarget as HTMLElement & { value: string }

        value = target.value
        onchange?.(value, event)
    }
</script>

<mdui-text-field
    {variant}
    {type}
    {icon}
    {placeholder}
    {clearable}
    value={value}
    class={className}
    oninput={handleInput}
    onchange={handleChange}
    {...props}
></mdui-text-field>