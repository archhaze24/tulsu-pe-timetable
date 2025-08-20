import { readable } from 'svelte/store'

export interface Teacher {
  id: number
  name: string
}

const initialTeachers: Teacher[] = [
  { id: 1, name: 'Иванов И.И.' },
  { id: 2, name: 'Петров П.П.' },
  { id: 3, name: 'Сидорова А.А.' },
  { id: 4, name: 'Кузнецов Н.Н.' },
  { id: 5, name: 'Смирнова О.О.' }
]

export const teachersStore = readable<Teacher[]>(initialTeachers)


