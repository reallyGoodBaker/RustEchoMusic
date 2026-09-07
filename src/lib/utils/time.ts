export function formatSeconds(
  seconds: number,
  options: {
    padMinutes?: boolean
    showHours?: boolean
  } = {}
): string {
  const { padMinutes = false, showHours = false } = options

  if (!Number.isFinite(seconds) || seconds <= 0) {
    return padMinutes ? '00:00' : '0:00'
  }

  const total = Math.floor(seconds)
  const h = Math.floor(total / 3600)
  const m = Math.floor((total % 3600) / 60)
  const s = total % 60

  if (h > 0 || showHours) {
    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  }

  const minutes = padMinutes ? String(m).padStart(2, '0') : String(m)

  return `${minutes}:${String(s).padStart(2, '0')}`
}

export function formatTime(seconds: number): string {
  return formatSeconds(seconds, { padMinutes: true })
}

export function formatDuration(seconds: number): string {
  return formatSeconds(seconds)
}