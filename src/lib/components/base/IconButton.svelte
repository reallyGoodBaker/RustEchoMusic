<script lang="ts">
    const {
        onclick = () => {},
        ...props
    } = $props()
</script>

<mdui-button-icon
    onclick={(event: MouseEvent) => onclick(event)}
    onkeydown={(event: KeyboardEvent) => {
        if (event.key === 'Enter' || event.key === ' ') {
            onclick(event)
        }
    }}
    role="button"
    tabindex="0"
    {...props}
></mdui-button-icon>