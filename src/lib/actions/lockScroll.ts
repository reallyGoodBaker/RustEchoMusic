export function lockScroll(node: HTMLElement) {
  const originalOverflow = document.body.style.overflow
  document.body.style.overflow = 'hidden'
  return {
    destroy() {
      document.body.style.overflow = originalOverflow
    },
  }
}
