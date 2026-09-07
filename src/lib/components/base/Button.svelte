<script lang="ts">
    const { onclick = () => {}, children, ...props } = $props()
</script>

<mdui-button
    onclick={() => onclick()}
    onkeydown={(e: KeyboardEvent) =>
        (e.key === 'Enter' || e.key === ' ') && onclick()}
    role="button"
    tabindex="0"
    {...props}
>
    {@render children()}
</mdui-button>
