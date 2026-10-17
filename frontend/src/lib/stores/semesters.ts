import { writable } from 'svelte/store'
import * as App from '../api'
import type { Semester, SemesterRequest } from '../api'
import { request, requestFlag } from './request'

export const semestersStore = writable<Semester[]>([])

// Бэкенд отдаёт список по убыванию даты начала; сортировка на фронте страхует порядок при любом источнике.
const byStartDesc = (a: Semester, b: Semester) => b.startDate.localeCompare(a.startDate)

// При ошибке список не очищается: показано предыдущее состояние и сообщение из request.
export async function loadSemesters(): Promise<boolean> {
  const data = await request('GetSemesters', () => App.GetSemesters())
  if (!data) return false
  semestersStore.set([...data].sort(byStartDesc))
  return true
}

export async function addSemester(req: Omit<SemesterRequest, 'id'>): Promise<Semester | null> {
  const created = await request('CreateSemester', () => App.CreateSemester(req))
  if (created) await loadSemesters()
  return created
}

export async function updateSemester(req: SemesterRequest): Promise<boolean> {
  const ok = await requestFlag('UpdateSemester', () => App.UpdateSemester(req))
  if (ok) await loadSemesters()
  return ok
}

export async function deleteSemester(id: number): Promise<boolean> {
  const ok = await requestFlag('DeleteSemester', () => App.DeleteSemester(id))
  if (ok) await loadSemesters()
  return ok
}

export function fetchArchivedSemesters(): Promise<Semester[] | null> {
  return request('GetSemestersByArchived', () => App.GetSemestersByArchived(true))
}

export async function restoreSemester(id: number): Promise<boolean> {
  const ok = await requestFlag('RestoreSemester', () => App.RestoreSemester(id))
  if (ok) await loadSemesters()
  return ok
}
