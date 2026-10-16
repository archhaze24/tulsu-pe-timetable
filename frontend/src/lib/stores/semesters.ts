import { writable, get } from 'svelte/store'
import * as App from '../api'
import { request } from './request'

export interface Semester {
  id: number
  name: string
  startDate: string
  endDate: string
}

export const semestersStore = writable<Semester[]>([])

// Даты приходят как строки с временем или без него; нужна только часть YYYY-MM-DD.
const onlyDate = (v: unknown): string => String(v ?? '').slice(0, 10)

const mapSemester = (s: App.Semester): Semester => ({
  id: Number(s.id),
  name: String(s.name),
  startDate: onlyDate(s.start_date),
  endDate: onlyDate(s.end_date)
})

export async function loadSemesters(): Promise<void> {
  const data = await request('GetSemesters', () => App.GetSemesters(), true)
  semestersStore.set((data ?? []).map(mapSemester))
}

export async function addSemester(data: Omit<Semester, 'id'>): Promise<Semester | null> {
  const created = await request('CreateSemester', () => App.CreateSemester({
    name: data.name,
    start_date: data.startDate,
    end_date: data.endDate
  }))
  if (!created) return null
  const mapped = mapSemester(created)
  semestersStore.update(list => [...list, mapped])
  return mapped
}

export async function updateSemester(id: number, changes: Partial<Omit<Semester, 'id'>>): Promise<boolean> {
  const current = get(semestersStore).find(s => s.id === id)
  if (!current) return false
  const next = { ...current, ...changes }
  // Бэкенд возвращает bool, а не обновлённую запись: обновляем стор локально.
  const ok = await request('UpdateSemester', () => App.UpdateSemester({
    id,
    name: next.name,
    start_date: next.startDate,
    end_date: next.endDate
  }))
  if (ok === undefined) return false
  semestersStore.update(list => list.map(s => (s.id === id ? next : s)))
  return Boolean(ok)
}


export async function deleteSemester(id: number): Promise<boolean> {
  const ok = await request('DeleteSemester', () => App.DeleteSemester(id))
  if (ok === undefined) return false
  semestersStore.update(list => list.filter(s => s.id !== id))
  return Boolean(ok)
}

export async function fetchArchivedSemesters(): Promise<Semester[]> {
  const data = await request('GetSemestersByArchived', () => App.GetSemestersByArchived(true), true)
  return (data ?? []).map(mapSemester)
}

export async function restoreSemester(id: number): Promise<boolean> {
  return Boolean(await request('RestoreSemester', () => App.RestoreSemester(id), true))
}
