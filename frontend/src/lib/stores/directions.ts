import { writable } from 'svelte/store'
import * as App from '../api'
import type { Direction, SaveDirectionRequest } from '../api'
import { request, requestFlag } from './request'
import { refreshTeachers } from './teachers'

export const directionsStore = writable<Direction[]>([])

// При ошибке список не очищается: показано предыдущее состояние и сообщение из request.
export async function refreshDirections(): Promise<boolean> {
  const data = await request('GetDirections', () => App.GetDirections())
  if (!data) return false
  directionsStore.set(data)
  return true
}

export function fetchArchivedDirections(): Promise<Direction[] | null> {
  return request('GetDirectionsByArchived', () => App.GetDirectionsByArchived(true))
}

// Архивация и восстановление меняют подпись направления у преподавателей, поэтому обновляем и их.
export async function restoreDirection(id: number): Promise<boolean> {
  const ok = await requestFlag('RestoreDirection', () => App.RestoreDirection(id))
  if (ok) await Promise.all([refreshDirections(), refreshTeachers()])
  return ok
}

export async function saveDirection(req: SaveDirectionRequest): Promise<boolean> {
  const saved = await request('SaveDirection', () => App.SaveDirection(req))
  if (!saved) return false
  await Promise.all([refreshDirections(), refreshTeachers()])
  return true
}

export async function deleteDirection(id: number): Promise<boolean> {
  const ok = await requestFlag('DeleteDirection', () => App.DeleteDirection(id))
  if (ok) await Promise.all([refreshDirections(), refreshTeachers()])
  return ok
}
