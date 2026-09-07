<script lang="ts">
    import 'mdui/components/button-icon.js'
    import 'mdui/components/button.js'
    import 'mdui/components/text-field.js'

    type Props = {
        value: string[]
        onchange: (newArr: string[]) => void
    }

    let { value, onchange }: Props = $props()

    function updateItem(index: number, newValue: string) {
        const arr = [...value]
        arr[index] = newValue
        onchange(arr)
    }

    function removeItem(index: number) {
        onchange(value.filter((_, i) => i !== index))
    }

    function addItem() {
        onchange([...value, ''])
    }
</script>

<div class="flex w-[280px] flex-col gap-2">
    {#each value as item, i (i)}
        <div class="flex items-center gap-1">
            <mdui-text-field
                value={item}
                onchange={(e: Event) => {
                    const target = e.currentTarget as HTMLInputElement
                    updateItem(i, target.value)
                }}
                role="textbox"
                tabindex="0"
                style="flex: 1;"
            ></mdui-text-field>
            <mdui-button-icon
                icon="delete"
                aria-label="删除该项"
                onclick={() => removeItem(i)}
                onkeydown={(e: KeyboardEvent) => {
                    if (e.key === 'Enter' || e.key === ' ') removeItem(i)
                }}
                role="button"
                tabindex="0"
            ></mdui-button-icon>
        </div>
    {/each}
    <mdui-button
        icon="add"
        onclick={addItem}
        onkeydown={(e: KeyboardEvent) => {
            if (e.key === 'Enter' || e.key === ' ') addItem()
        }}
        role="button"
        tabindex="0">添加</mdui-button>
</div>
