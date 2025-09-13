import { LogClientEvent } from '../../../wailsjs/go/app_services/App'
import { get } from 'svelte/store'
import { route } from './router'

export type FrontendPayload = Record<string, unknown>

export async function logUiEvent(name: string, payload: FrontendPayload = {}) {
  try {
    const r = get(route)
    const enriched: Record<string, unknown> = {
      ...payload,
      routeName: r?.name,
      routeParams: r?.params ?? undefined,
    }
    await LogClientEvent(name, enriched as any)
  } catch (e) {
    // swallow errors to avoid breaking UI; logging is best-effort
    console.warn('logUiEvent failed', e)
  }
}
