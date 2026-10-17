// Слои поверх окна (модалки, поповеры). Escape закрывает только верхний слой, фокус уходит внутрь диалога
// и возвращается на элемент, который был в фокусе при открытии.

export interface OverlayOptions {
  onclose: () => void
}

interface Layer {
  close: () => void
}

const layers: Layer[] = []

const FOCUSABLE =
  'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

export function overlay(node: HTMLElement, options: OverlayOptions) {
  const returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
  const layer: Layer = { close: options.onclose }
  layers.push(layer)

  const target = node.querySelector<HTMLElement>(FOCUSABLE) ?? node
  target.focus()

  const onKeydown = (e: KeyboardEvent) => {
    if (e.key !== 'Tab') return
    const items = node.querySelectorAll<HTMLElement>(FOCUSABLE)
    if (items.length === 0) return
    const head = items[0]
    const tail = items[items.length - 1]
    if (e.shiftKey && document.activeElement === head) {
      e.preventDefault()
      tail.focus()
    } else if (!e.shiftKey && document.activeElement === tail) {
      e.preventDefault()
      head.focus()
    }
  }
  node.addEventListener('keydown', onKeydown)

  return {
    update(next: OverlayOptions) {
      layer.close = next.onclose
    },
    destroy() {
      node.removeEventListener('keydown', onKeydown)
      layers.splice(layers.indexOf(layer), 1)
      returnFocus?.focus()
    },
  }
}

// Возвращает true, если закрыт слой; false — если открытых слоёв нет.
export function closeTopOverlay(): boolean {
  const top = layers.at(-1)
  if (!top) return false
  top.close()
  return true
}
