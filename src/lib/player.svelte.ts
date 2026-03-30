type PlayerState = {
  current: Track | null
  isPlaying: boolean
  playlist: Track[]
  progress: number
}

type Track = {
  title: string
  artist: string
  path: string
}

export const playerState = $state<PlayerState>({
  current: null,
  isPlaying: false,
  playlist: [],
  progress: 0,
})
