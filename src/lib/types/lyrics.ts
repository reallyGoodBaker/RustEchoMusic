export interface LyricLine {
    timestampMs: number
    text: string
}

export interface LyricDocument {
    songId: number
    lines: LyricLine[]
}
