import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

// Порт фиксированный: его ожидает Tauri (devUrl в src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true
  }
})
