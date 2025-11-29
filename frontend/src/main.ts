import './style.css'
import './lib/i18n'
import App from './App.svelte'
import { logUiEvent } from './lib/stores/logger'

const app = new App({
  target: document.getElementById('app')
})

window.addEventListener('click', (ev) => {
  const target = ev.target as HTMLElement | null
  if (!target) return

  const el = target.closest('button, [role="button"], [data-log-action]') as HTMLElement | null
  if (!el) return

  const action = el.getAttribute('data-log-action') || inferAction(el)
  const payload: Record<string, unknown> = {
    tag: el.tagName,
    id: el.id || undefined,
    classes: el.className || undefined,
    text: getElementText(el),
  }
  for (const [k, v] of Object.entries(el.dataset)) {
    payload[`data_${k}`] = v
  }
  void logUiEvent(action, payload)
}, { capture: true })

function inferAction(el: HTMLElement): string {
  const text = getElementText(el)
  if (text) return `click:${text}`
  return `click:${el.tagName.toLowerCase()}`
}

function getElementText(el: HTMLElement): string {
  const raw = (el.getAttribute('aria-label') || el.textContent || '').trim()
  return raw.replace(/\s+/g, ' ').slice(0, 120)
}

export default app
