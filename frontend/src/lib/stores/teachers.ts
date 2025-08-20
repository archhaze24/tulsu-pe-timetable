import { writable } from 'svelte/store'

export interface Teacher {
  id: number
  firstName: string
  lastName: string
  middleName?: string
  directionId: number
  rate: number
}

const initialTeachers: Teacher[] = [
  { id: 1, firstName: 'Иван', lastName: 'Иванов', middleName: 'Иванович', directionId: 1, rate: 1.0 },
  { id: 2, firstName: 'Пётр', lastName: 'Петров', middleName: 'Петрович', directionId: 1, rate: 0.5 },
  { id: 3, firstName: 'Анна', lastName: 'Сидорова', middleName: 'Алексеевна', directionId: 2, rate: 0.75 },
  { id: 4, firstName: 'Николай', lastName: 'Кузнецов', middleName: 'Николаевич', directionId: 3, rate: 1.0 },
]

export const teachersStore = writable<Teacher[]>(initialTeachers)

export function addTeacher(data: Omit<Teacher, 'id'>) {
  teachersStore.update(list => {
    const nextId = list.length ? Math.max(...list.map(t => t.id)) + 1 : 1
    return [...list, { id: nextId, ...data }]
  })
}

export function updateTeacher(id: number, changes: Partial<Omit<Teacher, 'id'>>) {
  teachersStore.update(list => list.map(t => (t.id === id ? { ...t, ...changes } : t)))
}

export function deleteTeacher(id: number) {
  teachersStore.update(list => list.filter(t => t.id !== id))
}

export function formatTeacherName(t: Teacher): string {
  const initials = [t.firstName, t.middleName]
    .filter(Boolean)
    .map(n => (n ? `${n[0]}.` : ''))
    .join(' ')
    .trim()
  return initials ? `${t.lastName} ${initials}` : t.lastName
}


