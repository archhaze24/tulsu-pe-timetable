import {defineConfig} from 'vite'
import {svelte} from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  // WebView2 Runtime 109 (последняя ветка для Windows 7) ≈ Chromium 109
  build: {
    target: 'chrome109',
  },
})
