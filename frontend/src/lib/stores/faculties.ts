import { writable } from 'svelte/store'

export interface Faculty {
  id: number
  name: string
}

const initialFaculties: Faculty[] = [
  { id: 1, name: 'Факультет математики' },
  { id: 2, name: 'Факультет информатики' },
  { id: 3, name: 'Факультет экономики' },
  { id: 4, name: 'Факультет менеджмента' },
]

export const facultiesStore = writable<Faculty[]>(initialFaculties)

export function updateFacultyName(id: number, name: string) {
  facultiesStore.update(list => list.map(f => (f.id === id ? { ...f, name } : f)))
}

export function addFaculty(name: string) {
  facultiesStore.update(list => {
    const nextId = list.length ? Math.max(...list.map(f => f.id)) + 1 : 1
    return [...list, { id: nextId, name }]
  })
}

export function deleteFaculty(id: number) {
  facultiesStore.update(list => list.filter(f => f.id !== id))
}


