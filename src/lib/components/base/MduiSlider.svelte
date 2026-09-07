<script lang="ts">
    import type { Slider } from 'mdui'
    import { onMount } from 'svelte'

    type MduiSliderElement = Slider & {
        value: number | string
        shadowRoot: ShadowRoot | null
    }

    type SliderProps = {
        nolabel?: boolean
        value?: number
        classList?: string
        class?: string
        duration?: number
        el?: Slider | null,
        oninput?: (event: Event) => void
        onchange?: (event: Event) => void
    }

    let {
        nolabel = false,
        value = $bindable(0),
        classList = '',
        class: className = '',
        oninput,
        onchange,
        duration = 0,
        el = $bindable(null)
    }: SliderProps = $props()

    let sliderRef = $state<MduiSliderElement | null>(null)

    const size = 1.5

    const styleText = `
        .handle .elevation,
        .handle::before {
            position: absolute;
            display: block;
            content: " ";
            left: ${size / 4}rem !important;
            top: ${size / 4}rem !important;
            width: ${size / 2}rem !important;
            height: ${size / 2}rem !important;
        }

        .handle {
            width: ${size}rem !important;
            height: ${size}rem !important;
            margin-top: ${-size / 2}rem !important;
        }
    `

    function updateSliderStyle(elem: MduiSliderElement | null): void {
        const shadow = elem?.shadowRoot
        if (!shadow) return

        const styleId = 'custom-slider-style'
        const existingStyle = shadow.querySelector<HTMLStyleElement>(
            `style[data-id="${styleId}"]`
        )

        if (existingStyle) {
            existingStyle.textContent = styleText
            return
        }

        const style = document.createElement('style')
        style.dataset.id = styleId
        style.textContent = styleText
        shadow.appendChild(style)
    }

    function getSliderValue(event: Event): number {
        const target = event.currentTarget as MduiSliderElement | null
        const rawValue = target?.value ?? 0
        const numericValue = Number(rawValue)

        return Number.isFinite(numericValue) ? numericValue : 0
    }

    function handleInput(event: Event): void {
        value = getSliderValue(event)
        oninput?.(event)
    }

    function handleChange(event: Event): void {
        value = getSliderValue(event)
        onchange?.(event)
    }

    onMount(() => {
        requestAnimationFrame(() => {
            updateSliderStyle(sliderRef)
        })
    })
</script>

<mdui-slider
    bind:this={el}
    {nolabel}
    max={duration}
    value={value}
    class={`${classList} ${className}`}
    oninput={handleInput}
    onchange={handleChange}
    bind:this={sliderRef}
></mdui-slider>