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

