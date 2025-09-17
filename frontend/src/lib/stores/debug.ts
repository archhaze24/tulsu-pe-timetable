import { writable } from 'svelte/store'

// Управляет видимостью дебаг-меню
export const debugMenuOpen = writable<boolean>(false)

export function openDebugMenu(): void {
  debugMenuOpen.set(true)
}

export function closeDebugMenu(): void {
  debugMenuOpen.set(false)
}

export function toggleDebugMenu(): void {
  debugMenuOpen.update((value) => !value)
}


