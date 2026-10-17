import { writable } from 'svelte/store'
import * as App from '../api'
import type { Teacher, UnbindResult } from '../api'
import { request, requestFlag } from './request'

export const semesterTeachersStore = writable<Teacher[]>([])

// При ошибке список не очищается: показано предыдущее состояние и сообщение из request.
export async function loadSemesterTeachers(semesterId: number): Promise<boolean> {
  const data = await request('GetAllTeachersForSemester', () => App.GetAllTeachersForSemester(semesterId))
  if (!data) return false
  semesterTeachersStore.set(data)
  return true
}

export async function bindTeacherToSemester(semesterId: number, teacherId: number): Promise<boolean> {
  const ok = await requestFlag('BindTeacherToSemester', () => App.BindTeacherToSemester({ semesterId, teacherId }))
  if (ok) await loadSemesterTeachers(semesterId)
  return ok
}

// Только расчёт: что снимется, без изменений в базе.
export function previewUnbindTeacher(semesterId: number, teacherId: number): Promise<UnbindResult | null> {
  return request('PreviewUnbindTeacher', () => App.PreviewUnbindTeacher({ semesterId, teacherId }))
}

export async function unbindTeacherFromSemester(semesterId: number, teacherId: number): Promise<UnbindResult | null> {
  const result = await request('UnbindTeacherFromSemester', () => App.UnbindTeacherFromSemester({ semesterId, teacherId }))
  if (result) await loadSemesterTeachers(semesterId)
  return result
}
