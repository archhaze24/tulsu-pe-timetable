import type { ApiResponse } from '../api'
import { notify } from './notify'

// Единая обработка ответа `{ data, error }`: возвращает данные или undefined при ошибке.
// Ошибка показывается уведомлением, кроме silent: тогда её покажет вызывающий код.
export async function request<T>(
  action: string,
  call: () => Promise<ApiResponse<T>>,
  silent = false
): Promise<T | undefined> {
  try {
    const resp = await call()
    if (resp.error) {
      console.error(`${action}: ${resp.error}`)
      if (!silent) notify(resp.error)
      return undefined
    }
    return resp.data
  } catch (e) {
    console.error(`${action} failed:`, e)
    if (!silent) notify(String(e))
    return undefined
  }
}
