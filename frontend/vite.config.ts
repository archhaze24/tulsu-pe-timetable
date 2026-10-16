import { defineConfig, type Plugin } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'
import { transform } from 'lightningcss'

// Порт фиксированный: его ожидает Tauri (devUrl в src-tauri/tauri.conf.json).
// `vite build --mode win7` собирает фронтенд для Windows 7 (WebView2 109), см. scripts/build-win7.mjs.
export default defineConfig(({ mode }) => {
  const win7 = mode === 'win7'
  return {
    plugins: [tailwindcss(), svelte(), ...(win7 ? [chrome109Css()] : [])],
    build: win7 ? { target: 'chrome109' } : undefined,
    clearScreen: false,
    server: {
      port: 5173,
      strictPort: true
    }
  }
})

// WebView2 109 (Chromium 109) — последняя версия для Windows 7. Tailwind 4 пишет цвета в oklch()
// и полупрозрачность через color-mix(in oklab, var(--color-…) …); в 109 это не работает, поэтому
// цвета понижаются до sRGB на этапе сборки. Обычная сборка не меняется.
const targets = { chrome: 109 << 16 }
const encoder = new TextEncoder()
const decoder = new TextDecoder()
const colorVar = /--(color-[\w-]+):\s*(#[\da-f]{3,8}|rgb\([^)]*\))(?=\s*[;}])/gi
const colorMixVar = /color-mix\(\s*in oklab\s*,\s*var\(--(color-[\w-]+)\)\s*([\d.]+%)\s*,\s*transparent\s*\)/g
// Фолбэк placeholder у Tailwind непрозрачный; у placeholder нет фона, поэтому opacity даёт тот же вид, что и 50% цвета.
const placeholderFallback = '::placeholder{color:currentColor}'
const placeholderHalf = '::placeholder{color:currentColor;opacity:.5}'

function lightning(css: string, minify: boolean): string {
  const { code } = transform({ filename: 'app.css', code: encoder.encode(css), minify, targets })
  return decoder.decode(code)
}

function lowerColorsForChrome109(css: string): string {
  const lowered = lightning(css, false)

  const palette = new Map<string, string>()
  for (const [, name, value] of lowered.matchAll(colorVar)) {
    if (!palette.has(name)) palette.set(name, value)
  }
  const inlined = lowered.replace(colorMixVar, (match, name: string, alpha: string) => {
    const value = palette.get(name)
    return value ? `color-mix(in oklab, ${value} ${alpha}, transparent)` : match
  })

  const result = lightning(inlined, true).split(placeholderFallback).join(placeholderHalf)
  if (/oklch\(|color-mix\([^)]*var\(/.test(result)) {
    throw new Error('Сборка для Windows 7: в CSS остались oklch() или color-mix() с var(), которые WebView2 109 не понимает')
  }
  return result
}

function chrome109Css(): Plugin {
  return {
    name: 'chrome109-css',
    apply: 'build',
    generateBundle(_options, bundle) {
      for (const file of Object.values(bundle)) {
        if (file.type === 'asset' && file.fileName.endsWith('.css')) {
          file.source = lowerColorsForChrome109(String(file.source))
        }
      }
    }
  }
}
