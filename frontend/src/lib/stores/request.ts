import { get } from 'svelte/store'
import { _ } from 'svelte-i18n'
import type { ApiResponse } from '../api'
import { notify } from './notify'

// Код ошибки бэкенда — ключ i18n. Неизвестный ключ (перевода нет) показываем общим текстом.
export function translateError(code: string): string {
  const translate = get(_)
  const text = translate(code)
  return text === code ? translate('errors.unknown') : text
}

// Единая обработка ответа `{ data, error }`: при ошибке показывает перевод кода и возвращает null.
export async function request<T>(action: string, call: () => Promise<ApiResponse<T>>): Promise<T | null> {
  try {
    const resp = await call()
    if (resp.error) {
      console.error(`${action}: ${resp.error}`)
      notify(translateError(resp.error))
      return null
    }
    return resp.data
  } catch (e) {
    console.error(`${action} failed:`, e)
    notify(get(_)('errors.unknown'))
    return null
  }
}

// Для команд, которые возвращают только признак успеха: true только при успехе.
export async function requestFlag(action: string, call: () => Promise<ApiResponse<boolean>>): Promise<boolean> {
  return (await request(action, call)) === true
}
