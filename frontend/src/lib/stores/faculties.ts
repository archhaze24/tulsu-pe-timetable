import { writable } from 'svelte/store'
import * as App from '../../../wailsjs/go/app_services/App.js'

export interface Faculty {
  id: number
  name: string
}

export const facultiesStore = writable<Faculty[]>([])

function mapFacultyFromBackend(f: any): Faculty {
  return { id: Number(f.id), name: String(f.name) }
}

export async function refreshFaculties(): Promise<void> {
  try {
    const resp = await App.GetFaculties()
    if (resp.error) {
      console.error('GetFaculties error:', resp.error)
      if (typeof window !== 'undefined') alert(resp.error)
      return
    }
    const mapped = (resp.data ?? []).map(mapFacultyFromBackend)
    facultiesStore.set(mapped)
  } catch (e) {
    console.error('GetFaculties failed:', e)
    if (typeof window !== 'undefined') alert(String(e))
  }
}

export async function fetchArchivedFaculties(): Promise<Faculty[]> {
  try {
    const resp = await App.GetFacultiesByArchived(true)
    if (resp.error) {
      console.error('GetFacultiesByArchived error:', resp.error)
      if (typeof window !== 'undefined') alert(resp.error)
      return []
    }
    return (resp.data ?? []).map(mapFacultyFromBackend)
  } catch (e) {
    console.error('GetFacultiesByArchived failed:', e)
    if (typeof window !== 'undefined') alert(String(e))
    return []
  }
}

export async function restoreFaculty(id: number): Promise<boolean> {
  try {
    const resp = await App.RestoreFaculty(id)
    if (resp.error) {
      console.error('RestoreFaculty error:', resp.error)
      if (typeof window !== 'undefined') alert(resp.error)
      return false
    }
    await refreshFaculties()
    return Boolean(resp.data)
  } catch (e) {
    console.error('RestoreFaculty failed:', e)
    if (typeof window !== 'undefined') alert(String(e))
    return false
  }
}

export async function updateFacultyName(id: number, name: string): Promise<void> {
  try {
    const resp = await App.UpdateFaculty({ id, name })
    if (resp.error) {
      console.error('UpdateFaculty error:', resp.error)
      if (typeof window !== 'undefined') alert(resp.error)
      return
    }
    await refreshFaculties()
  } catch (e) {
    console.error('UpdateFaculty failed:', e)
    if (typeof window !== 'undefined') alert(String(e))
  }
}

export async function addFaculty(name: string): Promise<void> {
  try {
    const resp = await App.CreateFaculty({ name })
    if (resp.error) {
      console.error('CreateFaculty error:', resp.error)
      if (typeof window !== 'undefined') alert(resp.error)
      return
    }
    await refreshFaculties()
  } catch (e) {
    console.error('CreateFaculty failed:', e)
    if (typeof window !== 'undefined') alert(String(e))
  }
}

export async function deleteFaculty(id: number): Promise<void> {
  try {
    const resp = await App.DeleteFaculty(id)
    if (resp.error) {
      console.error('DeleteFaculty error:', resp.error)
      if (typeof window !== 'undefined') alert(resp.error)
      return
    }
    await refreshFaculties()
  } catch (e) {
    console.error('DeleteFaculty failed:', e)
    if (typeof window !== 'undefined') alert(String(e))
  }
}

void refreshFaculties()

