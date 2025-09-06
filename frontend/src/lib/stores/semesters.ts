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

export interface TimeSlot {
  id: number
  semesterId: number
  dayOfWeek: DayOfWeek
  startTime: string
  endTime: string
}

const initialSemesters: Semester[] = [
  { id: 1, name: 'Осенний семестр 2025/2026', startDate: '2025-09-01', endDate: '2025-12-31' },
  { id: 2, name: 'Весенний семестр 2025/2026', startDate: '2026-02-01', endDate: '2026-06-30' }
]

export const semestersStore = writable<Semester[]>(initialSemesters)

const initialLessons: Lesson[] = [
  { id: 1, semesterId: 1, dayOfWeek: 1, startTime: '09:40', endTime: '11:15', directionId: 1, teacherIds: [1], facultyIds: [1], teacherCount: 1 },
  { id: 2, semesterId: 1, dayOfWeek: 1, startTime: '11:30', endTime: '13:10', directionId: 2, teacherIds: [3], facultyIds: [2], teacherCount: 1 },
  { id: 3, semesterId: 1, dayOfWeek: 3, startTime: '09:40', endTime: '11:15', directionId: 3, teacherIds: [4], facultyIds: [3], teacherCount: 1 }
]

export const lessonsStore = writable<Lesson[]>(initialLessons)

const initialTimeSlots: TimeSlot[] = [
  { id: 1, semesterId: 1, dayOfWeek: 1, startTime: '09:40', endTime: '11:15' },
  { id: 2, semesterId: 1, dayOfWeek: 1, startTime: '11:30', endTime: '13:10' },
  { id: 3, semesterId: 1, dayOfWeek: 3, startTime: '09:40', endTime: '11:15' },
]

export const timeSlotsStore = writable<TimeSlot[]>(initialTimeSlots)

export function addSemester(data: Omit<Semester, 'id'>): Semester {
  const list = get(semestersStore)
  const nextId = list.length ? Math.max(...list.map(s => s.id)) + 1 : 1
  const semester: Semester = { id: nextId, ...data }
  semestersStore.set([...list, semester])
  return semester
}

export function updateSemester(id: number, changes: Partial<Omit<Semester, 'id'>>) {
  semestersStore.update(list => list.map(s => (s.id === id ? { ...s, ...changes } : s)))
}

export function deleteSemester(id: number) {
  semestersStore.update(list => list.filter(s => s.id !== id))
  lessonsStore.update(list => list.filter(l => l.semesterId !== id))
}

export function addLesson(data: Omit<Lesson, 'id'>) {
  lessonsStore.update(list => {
    const nextId = list.length ? Math.max(...list.map(l => l.id)) + 1 : 1
    return [...list, { id: nextId, ...data }]
  })
}

export function updateLesson(id: number, changes: Partial<Omit<Lesson, 'id'>>) {
  lessonsStore.update(list => list.map(l => (l.id === id ? { ...l, ...changes } : l)))
}

export function deleteLesson(id: number) {
  lessonsStore.update(list => list.filter(l => l.id !== id))
}

export function addTimeSlot(data: Omit<TimeSlot, 'id'>) {
  timeSlotsStore.update(list => {
    const nextId = list.length ? Math.max(...list.map(t => t.id)) + 1 : 1
    return [...list, { id: nextId, ...data }]
  })
}

export function deleteTimeSlot(id: number) {
  timeSlotsStore.update(list => list.filter(t => t.id !== id))
}

function mapSemesterFromBackend(s: any): Semester {
  return {
    id: Number(s.id),
    name: String(s.name),
    startDate: String(s.start_date ?? ''),
    endDate: String(s.end_date ?? '')
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


