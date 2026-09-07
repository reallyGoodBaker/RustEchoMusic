import type { LyricDocument, LyricLine } from '$lib/types/lyrics'

class LyricsState {
    currentLyrics = $state<LyricDocument | null>(null)
    currentLineIndex = $state<number>(-1)
    isOpen = $state(false)

    handleLyricsLoaded(songId: number, lines: LyricLine[]) {
        this.currentLyrics = { songId, lines }
        this.currentLineIndex = -1
    }

    updateCurrentTime(currentTimeMs: number) {
        if (!this.currentLyrics || this.currentLyrics.lines.length === 0) {
            this.currentLineIndex = -1
            return
        }

        let idx = -1
        for (let i = this.currentLyrics.lines.length - 1; i >= 0; i--) {
            if (this.currentLyrics.lines[i].timestampMs <= currentTimeMs) {
                idx = i
                break
            }
        }
        this.currentLineIndex = idx
    }

    clear() {
        this.currentLyrics = null
        this.currentLineIndex = -1
    }

    toggle() {
        this.isOpen = !this.isOpen
    }
}

export const lyrics = new LyricsState()
