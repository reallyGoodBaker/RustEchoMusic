<script lang="ts">
    type SliderProps = {
        value?: number
        min?: number
        max?: number
        step?: number
        vertical?: boolean
        disabled?: boolean
        cssStyle?: string
        width?: number
        height?: number
        thumbHeight?: number
        onmousedown?: (event: MouseEvent) => void
        onmouseover?: (event: MouseEvent | FocusEvent) => void
        onmousemove?: (event: MouseEvent) => void
        onchange?: (value: number) => void
    }

    let {
        value = $bindable(0),
        min = 0,
        max = 100,
        step = 0,
        vertical = false,
        disabled = false,
        cssStyle = '',
        width = 100,
        height = 12,
        thumbHeight = 18,
        onmousedown,
        onmouseover,
        onmousemove,
        onchange,
    }: SliderProps = $props()

    let startMove = $state(false)
    let dx = 0
    let dy = 0
    let inputEl: HTMLDivElement | null = null

    const range = $derived(max - min)
    const percent = $derived(clamp(((value - min) / range) * 100, 0, 100))
    const progressRaw = $derived(percent / 100)
    const scale = $derived((width - 4) / width)

    function clamp(val: number, min: number, max: number): number {
        return Math.min(max, Math.max(min, val))
    }

    function roundToStep(val: number): number {
        if (step <= 0) return val
        return Math.round(val / step) * step
    }

    function commit(next: number): void {
        const rounded = clamp(roundToStep(next), min, max)
        if (rounded !== value) {
            value = rounded
            onchange?.(rounded)
        }
    }

    function setValueFromPercent(pct: number): void {
        commit(min + (pct / 100) * range)
    }

    function mouseDown(e: MouseEvent): void {
        if (disabled || !inputEl) return

        startMove = true
        if (vertical) {
            dy = e.offsetY
            setValueFromPercent(clamp((1 - dy / inputEl.offsetHeight) * 100, 0, 100))
        } else {
            dx = e.offsetX
            setValueFromPercent(clamp((dx / inputEl.offsetWidth) * 100, 0, 100))
        }

        onmousedown?.(e)
    }

    function mouseMove(e: MouseEvent): void {
        e.stopPropagation()

        if (!startMove || !inputEl) return

        let pct: number
        if (vertical) {
            dy += e.movementY
            pct = clamp((1 - dy / inputEl.offsetHeight) * 100, 0, 100)
        } else {
            dx += e.movementX
            pct = clamp((dx / inputEl.offsetWidth) * 100, 0, 100)
        }
        setValueFromPercent(pct)

        onmousemove?.(e)
    }

    function handleKeydown(e: KeyboardEvent): void {
        if (disabled) return

        const stepSize = step > 0 ? step : range / 10
        if (e.key === 'ArrowUp' || e.key === 'ArrowRight') {
            e.preventDefault()
            commit(value + stepSize)
        } else if (e.key === 'ArrowDown' || e.key === 'ArrowLeft') {
            e.preventDefault()
            commit(value - stepSize)
        }
    }

    $effect(() => {
        if (!startMove) return

        const onGlobalMouseUp = (): void => {
            startMove = false
            dx = 0
            dy = 0
        }

        document.addEventListener('mousemove', mouseMove, {
            passive: true,
            capture: true
        })

        document.addEventListener('mouseup', onGlobalMouseUp)

        return () => {
            document.removeEventListener('mousemove', mouseMove, {
                capture: true
            })

            document.removeEventListener('mouseup', onGlobalMouseUp)
        }
    })
</script>

<div
    bind:this={inputEl}
    class="container"
    class:vertical
    class:disabled
    style="--width: {width}; --thumbHeight: {thumbHeight}; --scale: {scale}; {cssStyle}"
    onmousedown={mouseDown}
    {onmouseover}
    onfocus={onmouseover}
    onkeydown={handleKeydown}
    role="slider"
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value}
    aria-valuetext={`${Math.round(value)}`}
    tabindex={disabled ? -1 : 0}
>
    <div class="track" style="--height: {height}; --progress: {progressRaw};">
        <div class="thumb-box"></div>
    </div>

    <div class="divider" style="--progress: {progressRaw};"></div>

    <div class="label" class:active={startMove}>
        {Math.round(value)}
    </div>
</div>

<style>
    .container {
        --width: 100;
        --thumbHeight: 18;
        position: relative;
        cursor: pointer;
    }

    .container.disabled {
        opacity: 0.4;
        cursor: not-allowed;
        pointer-events: none;
    }

    .container:not(.vertical) {
        width: calc(1px * var(--width));
        height: calc(var(--thumbHeight) * 1px);
    }

    .container.vertical {
        width: calc(var(--thumbHeight) * 1px);
        height: calc(1px * var(--width));
    }

    .track {
        --progress: 0;
        --height: 12;
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        overflow: hidden;
        border-radius: 12px;
    }

    .container:not(.vertical) .track {
        width: calc(100% - 2px);
        height: calc(var(--height) * 1px);
    }

    .container.vertical .track {
        width: calc(var(--height) * 1px);
        height: calc(100% - 2px);
    }

    .container > *,
    .track > * {
        pointer-events: none;
    }

    .thumb-box {
        width: 100%;
        height: 100%;
        position: relative;
        display: flex;
        gap: 7px;
    }

    .container.vertical .thumb-box {
        flex-direction: column-reverse;
    }

    .thumb-box::before,
    .thumb-box::after {
        content: '';
        position: relative;
        box-sizing: border-box;
    }

    .container:not(.vertical) .thumb-box::before {
        left: 0;
        width: calc(var(--progress) * var(--width) * 1px - 3px);
        height: 100%;
        border-radius: 12px 4px 4px 12px;
        background-color: rgb(var(--mdui-color-primary));
    }

    .container:not(.vertical) .thumb-box::after {
        right: 0;
        width: calc((1 - var(--progress)) * var(--width) * 1px - 3px);
        height: 100%;
        border-radius: 4px 12px 12px 4px;
        background-color: rgb(
            var(--mdui-color-surface-container-highest-light)
        );
    }

    .container.vertical .thumb-box::before {
        bottom: 0;
        width: 100%;
        height: calc(var(--progress) * var(--width) * 1px - 3px);
        border-radius: 4px 4px 12px 12px;
        background-color: rgb(var(--mdui-color-primary));
    }

    .container.vertical .thumb-box::after {
        top: 0;
        width: 100%;
        height: calc((1 - var(--progress)) * var(--width) * 1px - 3px);
        border-radius: 12px 12px 4px 4px;
        background-color: rgb(
            var(--mdui-color-surface-container-highest-light)
        );
    }

    .divider {
        anchor-name: --slider-thumb;
        position: absolute;
        box-sizing: border-box;
        display: flex;
        border: solid 1.5px rgb(var(--mdui-color-primary));
        border-radius: 3px;
        pointer-events: none;
        transition: all 0s ease;
        z-index: 2;
    }

    .container:not(.vertical) .divider {
        top: 50%;
        left: calc(var(--progress) * var(--scale) * 100% + 2px);
        transform: translate(-50%, -50%);
        width: 1.5px;
        height: calc(var(--thumbHeight) * 1px);
    }

    .container.vertical .divider {
        left: 50%;
        bottom: calc(var(--progress) * var(--scale) * 100% + 2px);
        transform: translate(-50%, 50%);
        width: calc(var(--thumbHeight) * 1px);
        height: 1.5px;
    }

    .label {
        position: absolute;
        position-anchor: --slider-thumb;
        background-color: rgb(var(--mdui-color-primary));
        color: rgb(var(--mdui-color-on-primary));
        font-size: 11px;
        font-weight: 500;
        line-height: 16px;
        padding: 4px 8px;
        border-radius: 100px;
        white-space: nowrap;
        transition: transform 150ms cubic-bezier(0.2, 0, 0, 1);
        z-index: 3;
    }

    .container:not(.vertical) .label {
        bottom: anchor(top);
        left: anchor(center);
        margin-bottom: 12px;
        transform: translate(-50%, 4px) scale(0);
        transform-origin: bottom center;
    }

    .container:not(.vertical) .label.active {
        transform: translate(-50%, 0) scale(1);
    }

    .container.vertical .label {
        right: anchor(left);
        top: anchor(center);
        margin-right: 12px;
        transform: translate(-4px, -50%) scale(0);
        transform-origin: center right;
    }

    .container.vertical .label.active {
        transform: translate(0, -50%) scale(1);
    }
</style>
