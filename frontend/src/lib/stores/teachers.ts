import { writable } from 'svelte/store'
import * as App from '../../../wailsjs/go/app_services/App.js'

export interface Teacher {
  id: number
  firstName: string
  lastName: string
  middleName?: string
  directionId: number
  rate: number
}

export const teachersStore = writable<Teacher[]>([])

function mapTeacherFromBackend(t: any): Teacher {
  return {
    id: Number(t.id),
    firstName: String(t.first_name ?? ''),
    lastName: String(t.last_name ?? ''),
    middleName: t.middle_name ? String(t.middle_name) : undefined,
    directionId: Number(t.direction_id ?? 0),
    rate: Number(t.rate ?? 0)
  }
}

export async function refreshTeachers(): Promise<void> {
  try {
    const resp = await App.GetTeachers()
    if (resp.error) {
      console.error('GetTeachers error:', resp.error)
      return
    }
    const mapped = (resp.data ?? []).map(mapTeacherFromBackend)
    teachersStore.set(mapped)
  } catch (e) {
    console.error('GetTeachers failed:', e)
  }
}

export async function fetchArchivedTeachers(): Promise<Teacher[]> {
  try {
    const resp = await App.GetTeachersByArchived(true)
    if (resp.error) {
      console.error('GetTeachersByArchived error:', resp.error)
      return []
    }
    return (resp.data ?? []).map(mapTeacherFromBackend)
  } catch (e) {
    console.error('GetTeachersByArchived failed:', e)
    return []
  }
}

export async function restoreTeacher(id: number): Promise<boolean> {
  try {
    const resp = await App.RestoreTeacher(id)
    if (resp.error) {
      console.error('RestoreTeacher error:', resp.error)
      return false
    }
    await refreshTeachers()
    return Boolean(resp.data)
  } catch (e) {
    console.error('RestoreTeacher failed:', e)
    return false
  }
}

export async function addTeacher(data: Omit<Teacher, 'id'>): Promise<void> {
  try {
    const resp = await App.CreateTeacher({
      first_name: data.firstName,
      last_name: data.lastName,
      middle_name: data.middleName ?? '',
      direction_id: data.directionId,
      rate: data.rate
    })
    if (resp.error) {
      console.error('CreateTeacher error:', resp.error)
      return
    }
    await refreshTeachers()
  } catch (e) {
    console.error('CreateTeacher failed:', e)
  }
}

export async function updateTeacher(id: number, changes: Partial<Omit<Teacher, 'id'>>): Promise<void> {
  try {
    // Read the current teacher to merge changes for required fields
    let current: Teacher | undefined
    teachersStore.update(list => {
      current = list.find(t => t.id === id)
      return list
    })
    if (!current) return
    const payload = {
      id,
      first_name: changes.firstName ?? current.firstName,
      last_name: changes.lastName ?? current.lastName,
      middle_name: (changes.middleName ?? current.middleName ?? ''),
      direction_id: changes.directionId ?? current.directionId,
      rate: changes.rate ?? current.rate
    }
    const resp = await App.UpdateTeacher(payload)
    if (resp.error) {
      console.error('UpdateTeacher error:', resp.error)
      return
    }
    await refreshTeachers()
  } catch (e) {
    console.error('UpdateTeacher failed:', e)
  }
}

export async function deleteTeacher(id: number): Promise<void> {
  try {
    const resp = await App.DeleteTeacher(id)
    if (resp.error) {
      console.error('DeleteTeacher error:', resp.error)
      return
    }
    await refreshTeachers()
  } catch (e) {
    console.error('DeleteTeacher failed:', e)
  }
}

export function formatTeacherName(t: Teacher): string {
  const initials = [t.firstName, t.middleName]
    .filter(Boolean)
    .map(n => (n ? `${(n as string)[0]}.` : ''))
    .join(' ')
    .trim()
  return initials ? `${t.lastName} ${initials}` : t.lastName
}

void refreshTeachers()

