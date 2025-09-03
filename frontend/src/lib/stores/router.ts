import { writable } from 'svelte/store'

export type RouteName =
  | 'home'
  | 'faculties'
  | 'faculty_edit'
  | 'directions'
  | 'direction_edit'
  | 'teachers'
  | 'teacher_edit'
  | 'semesters'
  | 'semester_edit'
  | 'schedule'

export interface RouteState {
  name: RouteName
  params?: Record<string, unknown>
}

export const route = writable<RouteState>({ name: 'home' })

export function navigate(name: RouteName, params?: Record<string, unknown>) {
  route.set({ name, params })
}


