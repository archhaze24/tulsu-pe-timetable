import { writable } from 'svelte/store'

export type RouteName =
  | 'home'
  | 'faculties'
  | 'faculty_edit'
  | 'faculties_archive'
  | 'directions'
  | 'direction_edit'
  | 'directions_archive'
  | 'teachers'
  | 'teacher_edit'
  | 'teachers_archive'
  | 'semesters'
  | 'semester_edit'
  | 'semesters_archive'
  | 'schedule'

export interface RouteState {
  name: RouteName
  params?: Record<string, unknown>
}

export const route = writable<RouteState>({ name: 'home' })

export function navigate(name: RouteName, params?: Record<string, unknown>) {
  route.set({ name, params })
}


