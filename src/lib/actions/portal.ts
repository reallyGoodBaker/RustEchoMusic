export function portal(node: HTMLElement, target: HTMLElement | string = 'body') {
  let targetNode: HTMLElement | null

  if (typeof target === 'string') {
    targetNode = document.querySelector(target)
  } else {
    targetNode = target
  }

  if (targetNode) {
    targetNode.appendChild(node)
  } else {
    console.warn(`Target node "${target}" not found for portal`)
  }

  return {
    destroy() {
      if (node.parentNode) {
        node.parentNode.removeChild(node)
      }
    }
  }
}
