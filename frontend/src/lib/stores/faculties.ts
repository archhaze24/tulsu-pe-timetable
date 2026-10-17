import { writable } from 'svelte/store'
import * as App from '../api'
import type { Faculty } from '../api'
import { request, requestFlag } from './request'

export const facultiesStore = writable<Faculty[]>([])

// При ошибке список не очищается: показано предыдущее состояние и сообщение из request.
export async function refreshFaculties(): Promise<boolean> {
  const data = await request('GetFaculties', () => App.GetFaculties())
  if (!data) return false
  facultiesStore.set(data)
  return true
}

export function fetchArchivedFaculties(): Promise<Faculty[] | null> {
  return request('GetFacultiesByArchived', () => App.GetFacultiesByArchived(true))
}

export async function restoreFaculty(id: number): Promise<boolean> {
  const ok = await requestFlag('RestoreFaculty', () => App.RestoreFaculty(id))
  if (ok) await refreshFaculties()
  return ok
}

export async function updateFacultyName(id: number, name: string): Promise<boolean> {
  const ok = await requestFlag('UpdateFaculty', () => App.UpdateFaculty({ id, name }))
  if (ok) await refreshFaculties()
  return ok
}

export async function addFaculty(name: string): Promise<Faculty | null> {
  const created = await request('CreateFaculty', () => App.CreateFaculty({ name }))
  if (created) await refreshFaculties()
  return created
}

export async function deleteFaculty(id: number): Promise<boolean> {
  const ok = await requestFlag('DeleteFaculty', () => App.DeleteFaculty(id))
  if (ok) await refreshFaculties()
  return ok
}
