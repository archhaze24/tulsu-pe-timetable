import { writable } from 'svelte/store'

export type ThemeMode = 'light' | 'dark'

// Single source of truth for theme across the app
export const theme = writable<ThemeMode>('light')

export function setTheme(next: ThemeMode) {
  theme.set(next)
}

export function toggleTheme() {
  theme.update((cur) => (cur === 'dark' ? 'light' : 'dark'))
}


