import { writable } from 'svelte/store'
import * as App from '../api'
import { request } from './request'

export interface Direction {
  id: number
  name: string
}

export const directionsStore = writable<Direction[]>([])

const mapDirection = (d: App.Direction): Direction => ({ id: Number(d.id), name: String(d.name) })

export async function refreshDirections(): Promise<void> {
  const data = await request('GetDirections', () => App.GetDirections(), true)
  if (data) directionsStore.set(data.map(mapDirection))
}

export async function fetchArchivedDirections(): Promise<Direction[]> {
  const data = await request('GetDirectionsByArchived', () => App.GetDirectionsByArchived(true), true)
  return (data ?? []).map(mapDirection)
}

export async function restoreDirection(id: number): Promise<boolean> {
  const ok = await request('RestoreDirection', () => App.RestoreDirection(id), true)
  if (ok === undefined) return false
  await refreshDirections()
  return Boolean(ok)
}

export async function updateDirection(id: number, name: string): Promise<void> {
  if (await request('UpdateDirection', () => App.UpdateDirection({ id, name }), true)) await refreshDirections()
}

export async function createDirection(name: string): Promise<Direction | null> {
  const data = await request('CreateDirection', () => App.CreateDirection({ name }), true)
  if (!data) return null
  await refreshDirections()
  return mapDirection(data)
}

export async function deleteDirection(id: number): Promise<void> {
  if (await request('DeleteDirection', () => App.DeleteDirection(id), true)) await refreshDirections()
}
