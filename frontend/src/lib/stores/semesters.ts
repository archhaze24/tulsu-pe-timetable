import { writable, get } from 'svelte/store'
import * as App from '../../../wailsjs/go/app_services/App.js'

export interface Semester {
  id: number
  name: string
  startDate: string
  endDate: string
}

export type DayOfWeek = 1 | 2 | 3 | 4 | 5 | 6 | 7

export interface Lesson {
  id: number
  semesterId: number
  dayOfWeek: DayOfWeek
  startTime: string
  endTime: string
  directionId: number
  teacherIds: number[]
  facultyIds: number[]
  teacherCount?: number
}

export const semestersStore = writable<Semester[]>([])
export const lessonsStore = writable<Lesson[]>([])

function mapSemesterFromBackend(s: any): Semester {
  const normalizeDate = (v: any): string => {
    const str = String(v ?? '')
    if (!str) return ''
    // Accept formats like YYYY-MM-DD, YYYY-MM-DDTHH:mm:ssZ, or with space time part
    if (str.includes('T')) return str.split('T')[0]
    if (str.includes(' ')) return str.split(' ')[0]
    // Trim potential trailing zeros like YYYY-MM-DD00:00:00
    return str.slice(0, 10)
  }
  return {
    id: Number(s.id),
    name: String(s.name),
    startDate: normalizeDate(s.start_date),
    endDate: normalizeDate(s.end_date)
  }
}

function mapLessonFromBackend(l: any): Lesson {
  const normalizeTime = (v: any): string => {
    const str = String(v ?? '')
    if (!str) return ''
    // Reduce HH:MM:SS -> HH:MM
    const parts = str.split(':')
    if (parts.length >= 2) return `${parts[0].padStart(2, '0')}:${parts[1].padStart(2, '0')}`
    return str
  }
  return {
    id: Number(l.id),
    semesterId: Number(l.semester_id),
    dayOfWeek: Number(l.day_of_week) as DayOfWeek,
    startTime: normalizeTime(l.start_time),
    endTime: normalizeTime(l.end_time),
    directionId: Number(l.direction_id ?? 0),
    teacherIds: Array.isArray(l.teacher_ids) ? (l.teacher_ids as any[]).map(Number) : [],
    facultyIds: Array.isArray(l.faculty_ids) ? (l.faculty_ids as any[]).map(Number) : [],
    teacherCount: typeof l.teacher_count === 'number' ? l.teacher_count : undefined
  }
}

// Semesters API-backed actions
export async function loadSemesters(): Promise<void> {
  try {
    const resp = await App.GetSemesters()
    if (resp.error) {
      console.error('GetSemesters error:', resp.error)
      semestersStore.set([])
      return
    }
    semestersStore.set((resp.data ?? []).map(mapSemesterFromBackend))
  } catch (e) {
    console.error('GetSemesters failed:', e)
    semestersStore.set([])
  }
}

export async function addSemester(data: Omit<Semester, 'id'>): Promise<Semester | null> {
  try {
    const resp = await App.CreateSemester({
      name: data.name,
      start_date: data.startDate,
      end_date: data.endDate
    })
    if (resp.error) {
      console.error('CreateSemester error:', resp.error)
      return null
    }
    const created = mapSemesterFromBackend(resp.data)
    semestersStore.update(list => [...list, created])
    return created
  } catch (e) {
    console.error('CreateSemester failed:', e)
    return null
  }
}

export async function updateSemester(id: number, changes: Partial<Omit<Semester, 'id'>>): Promise<Semester | null> {
  const current = get(semestersStore).find(s => s.id === id)
  if (!current) return null
  const next = { ...current, ...changes }
  try {
    const resp = await App.UpdateSemester({
      id: id,
      name: next.name,
      start_date: next.startDate,
      end_date: next.endDate
    })
    if (resp.error) {
      console.error('UpdateSemester error:', resp.error)
      return null
    }
    const updated = mapSemesterFromBackend(resp.data)
    semestersStore.update(list => list.map(s => (s.id === id ? updated : s)))
    return updated
  } catch (e) {
    console.error('UpdateSemester failed:', e)
    return null
  }
}

export async function deleteSemester(id: number): Promise<boolean> {
  try {
    const resp = await App.DeleteSemester(id)
    if (resp.error) {
      console.error('DeleteSemester error:', resp.error)
      return false
    }
    semestersStore.update(list => list.filter(s => s.id !== id))
    lessonsStore.update(list => list.filter(l => l.semesterId !== id))
    return Boolean(resp.data)
  } catch (e) {
    console.error('DeleteSemester failed:', e)
    return false
  }
}

export async function fetchArchivedSemesters(): Promise<Semester[]> {
  try {
    const resp = await App.GetSemestersByArchived(true)
    if (resp.error) {
      console.error('GetSemestersByArchived error:', resp.error)
      return []
    }
    return (resp.data ?? []).map(mapSemesterFromBackend)
  } catch (e) {
    console.error('GetSemestersByArchived failed:', e)
    return []
  }
}

export async function restoreSemester(id: number): Promise<boolean> {
  try {
    const resp = await App.RestoreSemester(id)
    if (resp.error) {
      console.error('RestoreSemester error:', resp.error)
      return false
    }
    return Boolean(resp.data)
  } catch (e) {
    console.error('RestoreSemester failed:', e)
    return false
  }
}

// Lessons API-backed actions
export async function loadLessonsBySemester(semesterId: number): Promise<void> {
  if (!semesterId) return
  try {
    const resp = await App.GetLessonsBySemester(semesterId)
    if (resp.error) {
      console.error('GetLessonsBySemester error:', resp.error)
      // Clear only that semester's lessons
      lessonsStore.update(list => list.filter(l => l.semesterId !== semesterId))
      return
    }
    const mapped = (resp.data ?? []).map(mapLessonFromBackend)
    lessonsStore.update(list => [
      ...list.filter(l => l.semesterId !== semesterId),
      ...mapped
    ])
  } catch (e) {
    console.error('GetLessonsBySemester failed:', e)
    lessonsStore.update(list => list.filter(l => l.semesterId !== semesterId))
  }
}

export async function addLesson(data: Omit<Lesson, 'id'>): Promise<Lesson | null> {
  try {
    const resp = await App.CreateLesson({
      semester_id: data.semesterId,
      day_of_week: data.dayOfWeek,
      start_time: data.startTime,
      end_time: data.endTime,
      direction_id: data.directionId,
      teacher_count: data.teacherCount,
      faculty_ids: data.facultyIds ?? [],
      teacher_ids: data.teacherIds ?? []
    })
    if (resp.error) {
      console.error('CreateLesson error:', resp.error)
      return null
    }
    const created = mapLessonFromBackend(resp.data)
    lessonsStore.update(list => [...list, created])
    return created
  } catch (e) {
    console.error('CreateLesson failed:', e)
    return null
  }
}

export async function updateLesson(id: number, changes: Partial<Omit<Lesson, 'id'>>): Promise<Lesson | null> {
  const current = get(lessonsStore).find(l => l.id === id)
  if (!current) return null
  const next = { ...current, ...changes }
  try {
    const resp = await App.UpdateLesson({
      id: next.id,
      semester_id: next.semesterId,
      day_of_week: next.dayOfWeek,
      start_time: next.startTime,
      end_time: next.endTime,
      direction_id: next.directionId,
      teacher_count: next.teacherCount,
      faculty_ids: next.facultyIds ?? [],
      teacher_ids: next.teacherIds ?? []
    })
    if (resp.error) {
      console.error('UpdateLesson error:', resp.error)
      return null
    }
    const updated = mapLessonFromBackend(resp.data)
    lessonsStore.update(list => list.map(l => (l.id === id ? updated : l)))
    return updated
  } catch (e) {
    console.error('UpdateLesson failed:', e)
    return null
  }
}

export async function deleteLesson(id: number): Promise<boolean> {
  try {
    const resp = await App.DeleteLesson(id)
    if (resp.error) {
      console.error('DeleteLesson error:', resp.error)
      return false
    }
    lessonsStore.update(list => list.filter(l => l.id !== id))
    return Boolean(resp.data)
  } catch (e) {
    console.error('DeleteLesson failed:', e)
    return false
  }
}

