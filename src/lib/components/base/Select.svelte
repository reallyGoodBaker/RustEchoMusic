<script lang="ts">
    import 'mdui/components/menu-item.js'
    import 'mdui/components/select.js'

    export type SelectOption = {
        label: string
        value: string
    }

    type SelectProps = {
        value?: string
        options: SelectOption[]
        placeholder?: string
        class?: string
        onchange?: (value: string, event: Event) => void
        [key: string]: unknown
    }

    let {
        value = $bindable(''),
        options,
        placeholder,
        class: className = '',
        onchange,
        ...props
    }: SelectProps = $props()

    let opened = $state(false)

    function handleChange(event: Event) {
        const target = event.currentTarget as HTMLElement & { value: string }

        value = target.value
        opened = false
        onchange?.(value, event)
    }

    function handleClick() {
        opened = !opened
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Enter' || event.key === ' ') {
            opened = !opened
        }

        if (event.key === 'Escape') {
            opened = false
        }
    }
</script>

<div class={className}>
    <mdui-select
        {value}
        {placeholder}
        end-icon={opened
            ? 'arrow_drop_up--rounded'
            : 'arrow_drop_down--rounded'}
        onchange={handleChange}
        onclick={handleClick}
        onkeydown={handleKeydown}
        onblur={() => (opened = false)}
        aria-expanded={opened}
        {...props}
    >
        {#each options as option (option.value)}
            <mdui-menu-item value={option.value}>
                {option.label}
            </mdui-menu-item>
        {/each}
    </mdui-select>
</div>
