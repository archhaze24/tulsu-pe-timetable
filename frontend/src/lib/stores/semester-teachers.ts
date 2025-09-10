import { writable } from 'svelte/store'
import * as App from '../../../wailsjs/go/app_services/App.js'

export interface SemesterTeacher {
  id: number
  firstName: string
  lastName: string
  middleName?: string
  directionId: number
  directionName?: string
  rate: number
  isArchived: boolean
  isGuest: boolean
  isBound: boolean
}

export const semesterTeachersStore = writable<SemesterTeacher[]>([])

function mapFromBackend(t: any): SemesterTeacher {
  return {
    id: Number(t.id),
    firstName: String(t.first_name ?? ''),
    lastName: String(t.last_name ?? ''),
    middleName: t.middle_name ? String(t.middle_name) : undefined,
    directionId: Number(t.direction_id ?? 0),
    directionName: t.direction_name ? String(t.direction_name) : undefined,
    rate: Number(t.rate ?? 0),
    isArchived: Boolean(t.is_archived ?? t.isArchived ?? false),
    isGuest: Boolean(t.is_guest ?? t.isGuest ?? false),
    isBound: Boolean(t.is_bound ?? t.isBound ?? false)
  }
}

export async function loadSemesterTeachers(semesterId: number): Promise<void> {
  try {
    const resp = await App.GetAllTeachersForSemester(semesterId)
    if (resp.error) {
      console.error('GetAllTeachersForSemester error:', resp.error)
      semesterTeachersStore.set([])
      return
    }
    const mapped = (resp.data ?? []).map(mapFromBackend)
    semesterTeachersStore.set(mapped)
  } catch (e) {
    console.error('GetAllTeachersForSemester failed:', e)
    semesterTeachersStore.set([])
  }
}

export async function bindTeacherToSemester(semesterId: number, teacherId: number): Promise<boolean> {
  try {
    const resp = await App.BindTeacherToSemester({ semester_id: semesterId, teacher_id: teacherId })
    if (resp.error) {
      console.error('BindTeacherToSemester error:', resp.error)
      return false
    }
    await loadSemesterTeachers(semesterId)
    return Boolean(resp.data)
  } catch (e) {
    console.error('BindTeacherToSemester failed:', e)
    return false
  }
}

export async function unbindTeacherFromSemester(semesterId: number, teacherId: number): Promise<boolean> {
  try {
    const resp = await App.UnbindTeacherFromSemester({ semester_id: semesterId, teacher_id: teacherId })
    if (resp.error) {
      console.error('UnbindTeacherFromSemester error:', resp.error)
      return false
    }
    await loadSemesterTeachers(semesterId)
    return Boolean(resp.data)
  } catch (e) {
    console.error('UnbindTeacherFromSemester failed:', e)
    return false
  }
}


