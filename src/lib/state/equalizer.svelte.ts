import { invoke } from '@tauri-apps/api/core'

// 10 段 EQ 的中心频率（Hz）
const EQ_BANDS = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000]
// 与 EQ_BANDS 对应的显示标签
const BAND_LABELS = ['31', '62', '125', '250', '500', '1k', '2k', '4k', '8k', '16k']

export type EqPreset = {
    name: string
    bands: number[]
}

export const EQ_PRESETS: EqPreset[] = [
    { name: 'Flat', bands: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { name: 'Bass Boost', bands: [6, 5, 4, 2, 0, 0, 0, 0, 0, 0] },
    { name: 'Treble Boost', bands: [0, 0, 0, 0, 0, 2, 4, 5, 6, 6] },
    { name: 'Vocal', bands: [-3, -2, 0, 2, 4, 4, 3, 1, 0, -1] },
]

class EqualizerState {
    bands = $state<number[]>([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    enabled = $state(true)
    presetName = $state('Flat')
    isOpen = $state(false)
    loaded = $state(false)

    async load() {
        try {
            const state = await invoke<{ bands: number[]; enabled: boolean; preset_name: string }>('get_eq_state')
            this.bands = state.bands
            this.enabled = state.enabled
            this.presetName = state.preset_name
            this.loaded = true
        } catch (err) {
            console.error('Failed to load EQ state:', err)
        }
    }

    async setBand(index: number, gain: number) {
        this.bands[index] = gain
        try {
            await invoke('set_eq_band', { band: index, gain })
        } catch (err) {
            console.error(err)
        }
    }

    async applyPreset(preset: EqPreset) {
        this.bands = [...preset.bands]
        this.presetName = preset.name
        try {
            await invoke('apply_eq_preset', { presetName: preset.name, bands: preset.bands })
        } catch (err) {
            console.error(err)
        }
    }

    async setEnabled(enabled: boolean) {
        this.enabled = enabled
        try {
            await invoke('set_eq_enabled', { enabled })
        } catch (err) {
            console.error(err)
        }
    }

    toggle() {
        this.isOpen = !this.isOpen
    }
}

export const equalizer = new EqualizerState()
export { EQ_BANDS, BAND_LABELS }
