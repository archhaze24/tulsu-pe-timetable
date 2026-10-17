import { writable } from 'svelte/store'
import * as App from '../api'
import type { Teacher, TeacherRequest } from '../api'
import { request, requestFlag } from './request'

export const teachersStore = writable<Teacher[]>([])

// При ошибке список не очищается: показано предыдущее состояние и сообщение из request.
export async function refreshTeachers(): Promise<boolean> {
  const data = await request('GetTeachers', () => App.GetTeachers())
  if (!data) return false
  teachersStore.set(data)
  return true
}

export function fetchArchivedTeachers(): Promise<Teacher[] | null> {
  return request('GetTeachersByArchived', () => App.GetTeachersByArchived(true))
}

export async function restoreTeacher(id: number): Promise<boolean> {
  const ok = await requestFlag('RestoreTeacher', () => App.RestoreTeacher(id))
  if (ok) await refreshTeachers()
  return ok
}

export async function addTeacher(req: Omit<TeacherRequest, 'id'>): Promise<Teacher | null> {
  const created = await request('CreateTeacher', () => App.CreateTeacher(req))
  if (created) await refreshTeachers()
  return created
}

export async function updateTeacher(req: TeacherRequest): Promise<boolean> {
  const ok = await requestFlag('UpdateTeacher', () => App.UpdateTeacher(req))
  if (ok) await refreshTeachers()
  return ok
}

export async function deleteTeacher(id: number): Promise<boolean> {
  const ok = await requestFlag('DeleteTeacher', () => App.DeleteTeacher(id))
  if (ok) await refreshTeachers()
  return ok
}

export function formatTeacherName(t: Pick<Teacher, 'firstName' | 'lastName' | 'middleName'>): string {
  const initials = [t.firstName, t.middleName]
    .filter(Boolean)
    .map(n => `${n[0]}.`)
    .join(' ')
  return initials ? `${t.lastName} ${initials}` : t.lastName
}
