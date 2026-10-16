import { writable } from 'svelte/store'

export const debugMenuOpen = writable<boolean>(false)

export function toggleDebugMenu(): void {
  debugMenuOpen.update((value) => !value)
}

export function closeDebugMenu(): void {
  debugMenuOpen.set(false)
}
