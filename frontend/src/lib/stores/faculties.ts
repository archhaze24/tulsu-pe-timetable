import { writable } from 'svelte/store'
import * as App from '../api'
import { request } from './request'

export interface Faculty {
  id: number
  name: string
}

export const facultiesStore = writable<Faculty[]>([])

const mapFaculty = (f: App.Faculty): Faculty => ({ id: Number(f.id), name: String(f.name) })

export async function refreshFaculties(): Promise<void> {
  const data = await request('GetFaculties', () => App.GetFaculties())
  if (data) facultiesStore.set(data.map(mapFaculty))
}

export async function fetchArchivedFaculties(): Promise<Faculty[]> {
  const data = await request('GetFacultiesByArchived', () => App.GetFacultiesByArchived(true))
  return (data ?? []).map(mapFaculty)
}

export async function restoreFaculty(id: number): Promise<boolean> {
  const ok = await request('RestoreFaculty', () => App.RestoreFaculty(id))
  if (ok === undefined) return false
  await refreshFaculties()
  return Boolean(ok)
}

export async function updateFacultyName(id: number, name: string): Promise<void> {
  if (await request('UpdateFaculty', () => App.UpdateFaculty({ id, name }))) await refreshFaculties()
}

export async function addFaculty(name: string): Promise<void> {
  if (await request('CreateFaculty', () => App.CreateFaculty({ name }))) await refreshFaculties()
}

export async function deleteFaculty(id: number): Promise<void> {
  if (await request('DeleteFaculty', () => App.DeleteFaculty(id))) await refreshFaculties()
}
