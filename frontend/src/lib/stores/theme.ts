import { writable } from 'svelte/store'

export type ThemeMode = 'light' | 'dark'

export const theme = writable<ThemeMode>('light')

export function setTheme(next: ThemeMode) {
  theme.set(next)
}
