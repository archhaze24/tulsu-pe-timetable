import { writable } from 'svelte/store'

export type NotifyKind = 'error' | 'info'

export interface Notice {
  id: number
  kind: NotifyKind
  message: string
}

const TIMEOUT_MS = 4000

export const notices = writable<Notice[]>([])

let nextId = 0

export function notify(message: string, kind: NotifyKind = 'error'): void {
  const id = ++nextId
  notices.update(list => [...list, { id, kind, message }])
  setTimeout(() => dismiss(id), TIMEOUT_MS)
}

export function dismiss(id: number): void {
  notices.update(list => list.filter(n => n.id !== id))
}
