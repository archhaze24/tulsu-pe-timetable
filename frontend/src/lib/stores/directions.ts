import { writable } from 'svelte/store'

export interface Direction {
  id: number
  name: string
  teacherIds: number[]
}

const initialDirections: Direction[] = [
  { id: 1, name: 'Футбол', teacherIds: [1, 2] },
  { id: 2, name: 'Плавание', teacherIds: [3] },
  { id: 3, name: 'Волейбол', teacherIds: [] }
]

export const directionsStore = writable<Direction[]>(initialDirections)

export function updateDirection(id: number, changes: Partial<Omit<Direction, 'id'>>) {
  directionsStore.update(list => list.map(d => (d.id === id ? { ...d, ...changes } : d)))
}

export function addDirection(name: string) {
  directionsStore.update(list => {
    const nextId = list.length ? Math.max(...list.map(d => d.id)) + 1 : 1
    return [...list, { id: nextId, name, teacherIds: [] }]
  })
}

export function deleteDirection(id: number) {
  directionsStore.update(list => list.filter(d => d.id !== id))
}

export function toggleTeacher(directionId: number, teacherId: number) {
  directionsStore.update(list => list.map(d => {
    if (d.id !== directionId) return d
    const exists = d.teacherIds.includes(teacherId)
    return { ...d, teacherIds: exists ? d.teacherIds.filter(id => id !== teacherId) : [...d.teacherIds, teacherId] }
  }))
}


