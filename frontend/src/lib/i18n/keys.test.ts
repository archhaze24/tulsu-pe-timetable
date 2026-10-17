/// <reference types="node" />
// TypeScript 6 не подключает @types/node по умолчанию, а тест читает файлы через node:fs.
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import ru from '../locales/ru.json'

const srcDir = fileURLToPath(new URL('../..', import.meta.url))

// Вызовы перевода: $t(...), $_(...), t(...), _(...), translate(...), get(_)(...).
const TRANSLATE_CALL = /(?<![\w$.])(?:\$t|\$_|t|_|translate|get\(_\))\(/g
// Строковые литералы без подстановок `${...}`: динамические ключи пропускаем.
const LITERAL = /'([^'\\]*)'|"([^"\\]*)"|`([^`$\\]*)`/g

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap(name => {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) return name === 'generated' ? [] : sourceFiles(path)
    return /\.(ts|svelte)$/.test(name) && !name.endsWith('.test.ts') ? [path] : []
  })
}

// Текст первого аргумента вызова: до запятой или закрывающей скобки на верхнем уровне.
function firstArgument(src: string, open: number): string {
  let depth = 0
  let quote = ''
  for (let i = open + 1; i < src.length; i++) {
    const ch = src[i]
    if (quote) {
      if (ch === '\\') i++
      else if (ch === quote) quote = ''
    } else if (ch === "'" || ch === '"' || ch === '`') {
      quote = ch
    } else if (ch === '(' || ch === '[' || ch === '{') {
      depth++
    } else if (ch === ')' || ch === ']' || ch === '}') {
      if (depth === 0) return src.slice(open + 1, i)
      depth--
    } else if (ch === ',' && depth === 0) {
      return src.slice(open + 1, i)
    }
  }
  return ''
}

// Ключ -> файлы, где он используется в литералах вызова перевода.
function collectKeys(): Map<string, Set<string>> {
  const found = new Map<string, Set<string>>()
  for (const file of sourceFiles(srcDir)) {
    const src = readFileSync(file, 'utf8')
    for (const call of src.matchAll(TRANSLATE_CALL)) {
      const arg = firstArgument(src, call.index + call[0].length - 1)
      for (const literal of arg.matchAll(LITERAL)) {
        const key = literal[1] ?? literal[2] ?? literal[3]
        if (!key) continue
        const files = found.get(key) ?? new Set<string>()
        files.add(file.slice(srcDir.length))
        found.set(key, files)
      }
    }
  }
  return found
}

function hasText(key: string): boolean {
  const value = key.split('.').reduce<unknown>(
    (node, part) => (node && typeof node === 'object' ? (node as Record<string, unknown>)[part] : undefined),
    ru,
  )
  return typeof value === 'string' && value.trim() !== ''
}

function emptyLeaves(node: unknown, path = ''): string[] {
  if (typeof node === 'string') return node.trim() ? [] : [path]
  if (node && typeof node === 'object') {
    return Object.entries(node).flatMap(([key, value]) => emptyLeaves(value, path ? `${path}.${key}` : key))
  }
  return [path]
}

describe('ключи i18n', () => {
  const used = collectKeys()

  it('сканер находит ключи в коде', () => {
    expect(used.size).toBeGreaterThan(50)
  })

  it('каждый ключ из литералов в коде есть в ru.json', () => {
    const missing = [...used]
      .filter(([key]) => !hasText(key))
      .map(([key, files]) => `${key} (${[...files].join(', ')})`)

    expect(missing).toEqual([])
  })

  // Ключи `outcome.${code}` динамические: проверяем их по списку кодов ScheduleOutcome.
  it('каждый код ScheduleOutcome есть в outcome', () => {
    const text = readFileSync(join(srcDir, 'lib/generated/ScheduleOutcome.ts'), 'utf8')
    // `ok` — успешный результат, текста ошибки для него нет.
    const codes = [...text.matchAll(/"code":\s*"(\w+)"/g)].map(([, code]) => code).filter(code => code !== 'ok')

    expect(codes.length).toBeGreaterThan(0)
    expect(codes.filter(code => !hasText(`outcome.${code}`))).toEqual([])
  })

  it('все строки ru.json непустые', () => {
    expect(emptyLeaves(ru)).toEqual([])
  })
})
