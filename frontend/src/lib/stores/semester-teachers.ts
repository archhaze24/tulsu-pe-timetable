import { writable } from 'svelte/store'
import * as App from '../api'
import type { SemesterTeacher } from '../api'
import { request } from './request'

export type { SemesterTeacher }

export const semesterTeachersStore = writable<SemesterTeacher[]>([])

export async function loadSemesterTeachers(semesterId: number): Promise<void> {
  const data = await request('GetAllTeachersForSemester', () => App.GetAllTeachersForSemester(semesterId), true)
  semesterTeachersStore.set(data ?? [])
}

export async function bindTeacherToSemester(semesterId: number, teacherId: number): Promise<boolean> {
  const ok = await request('BindTeacherToSemester', () => App.BindTeacherToSemester({ semester_id: semesterId, teacher_id: teacherId }), true)
  if (ok === undefined) return false
  await loadSemesterTeachers(semesterId)
  return Boolean(ok)
}

export async function unbindTeacherFromSemester(semesterId: number, teacherId: number): Promise<boolean> {
  const ok = await request('UnbindTeacherFromSemester', () => App.UnbindTeacherFromSemester({ semester_id: semesterId, teacher_id: teacherId }), true)
  if (ok === undefined) return false
  await loadSemesterTeachers(semesterId)
  return Boolean(ok)
}
