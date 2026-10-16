import { writable, get } from 'svelte/store'
import * as App from '../api'
import { request } from './request'

export interface Teacher {
  id: number
  firstName: string
  lastName: string
  middleName?: string
  directionId: number
  rate: number
  directionName?: string
}

export const teachersStore = writable<Teacher[]>([])

const mapTeacher = (t: App.Teacher): Teacher => ({
  id: Number(t.id),
  firstName: String(t.first_name ?? ''),
  lastName: String(t.last_name ?? ''),
  middleName: t.middle_name ? String(t.middle_name) : undefined,
  directionId: Number(t.direction_id ?? 0),
  rate: Number(t.rate ?? 0),
  directionName: t.direction_name ? String(t.direction_name) : undefined
})

export async function refreshTeachers(): Promise<void> {
  const data = await request('GetTeachers', () => App.GetTeachers(), true)
  if (data) teachersStore.set(data.map(mapTeacher))
}

export async function fetchArchivedTeachers(): Promise<Teacher[]> {
  const data = await request('GetTeachersByArchived', () => App.GetTeachersByArchived(true), true)
  return (data ?? []).map(mapTeacher)
}

export async function restoreTeacher(id: number): Promise<boolean> {
  const ok = await request('RestoreTeacher', () => App.RestoreTeacher(id), true)
  if (ok === undefined) return false
  await refreshTeachers()
  return Boolean(ok)
}

export async function addTeacher(data: Omit<Teacher, 'id'>): Promise<void> {
  const ok = await request('CreateTeacher', () => App.CreateTeacher({
    first_name: data.firstName,
    last_name: data.lastName,
    middle_name: data.middleName ?? '',
    direction_id: data.directionId,
    rate: data.rate
  }), true)
  if (ok !== undefined) await refreshTeachers()
}

export async function updateTeacher(id: number, changes: Partial<Omit<Teacher, 'id'>>): Promise<void> {
  const current = get(teachersStore).find(t => t.id === id)
  if (!current) return
  const ok = await request('UpdateTeacher', () => App.UpdateTeacher({
    id,
    first_name: changes.firstName ?? current.firstName,
    last_name: changes.lastName ?? current.lastName,
    middle_name: changes.middleName ?? current.middleName ?? '',
    direction_id: changes.directionId ?? current.directionId,
    rate: changes.rate ?? current.rate
  }), true)
  if (ok !== undefined) await refreshTeachers()
}

export async function deleteTeacher(id: number): Promise<void> {
  if (await request('DeleteTeacher', () => App.DeleteTeacher(id), true)) await refreshTeachers()
}

export function formatTeacherName(t: Teacher): string {
  const initials = [t.firstName, t.middleName]
    .filter(Boolean)
    .map(n => `${n![0]}.`)
    .join(' ')
  return initials ? `${t.lastName} ${initials}` : t.lastName
}
