// Тонкий слой над командами Tauri. Имена команд и форма ответа `{ data, error }` совпадают с командами в `src-tauri/src/commands.rs`.
// Типы моделей генерируются из Rust (`lib/generated`, см. `pnpm gen:types`).
import { invoke } from '@tauri-apps/api/core'
import type { Config } from './generated/Config'
import type { Direction } from './generated/Direction'
import type { Faculty } from './generated/Faculty'
import type { Teacher } from './generated/Teacher'
import type { Semester } from './generated/Semester'
import type { ScheduleOutcome } from './generated/ScheduleOutcome'
import type { ScheduleView } from './generated/ScheduleView'
import type { TeacherRequest } from './generated/TeacherRequest'
import type { SemesterRequest } from './generated/SemesterRequest'
import type { BindRequest } from './generated/BindRequest'
import type { SlotFacultiesRequest } from './generated/SlotFacultiesRequest'
import type { AddSlotRequest } from './generated/AddSlotRequest'
import type { DeleteSlotRequest } from './generated/DeleteSlotRequest'
import type { ToggleTeacherRequest } from './generated/ToggleTeacherRequest'
import type { UpdateConfigRequest } from './generated/UpdateConfigRequest'
import type { UpdateNameRequest } from './generated/UpdateNameRequest'
import type { NameRequest } from './generated/NameRequest'

import type { TeacherView } from './generated/TeacherView'

export type { Config, Direction, Faculty, Teacher, Semester, ScheduleOutcome, ScheduleView }
export type { SlotView } from './generated/SlotView'
export type { DayView } from './generated/DayView'
export type ScheduleTeacher = TeacherView

export interface ApiResponse<T> {
  data: T
  error: string
}

/** Преподаватель в контексте семестра: camelCase, как удобнее экрану (маппинг из `Teacher`). */
export interface SemesterTeacher {
  id: number
  firstName: string
  lastName: string
  middleName: string
  directionId: number
  directionName: string
  rate: number
  isArchived: boolean
  isGuest: boolean
  isBound: boolean
}

const call = <T>(command: string, args?: Record<string, unknown>): Promise<ApiResponse<T>> =>
  invoke<ApiResponse<T>>(command, args)

// ---------- Конфиг ----------

export const GetConfig = () => call<Config>('get_config')

export const UpdateConfig = (config: UpdateConfigRequest) =>
  call<boolean>('update_config', { config })

// ---------- Направления ----------

export const CreateDirection = (req: NameRequest) => call<Direction>('create_direction', { req })

export const GetDirections = () => call<Direction[]>('get_directions')

export const GetDirectionsByArchived = (isArchived: boolean) =>
  call<Direction[]>('get_directions_by_archived', { isArchived })

export const UpdateDirection = (req: UpdateNameRequest) => call<boolean>('update_direction', { req })

export const DeleteDirection = (id: number) => call<boolean>('delete_direction', { id })

export const RestoreDirection = (id: number) => call<boolean>('restore_direction', { id })

// ---------- Факультеты ----------

export const CreateFaculty = (req: NameRequest) => call<Faculty>('create_faculty', { req })

export const GetFaculties = () => call<Faculty[]>('get_faculties')

export const GetFacultiesByArchived = (isArchived: boolean) =>
  call<Faculty[]>('get_faculties_by_archived', { isArchived })

export const UpdateFaculty = (req: UpdateNameRequest) => call<boolean>('update_faculty', { req })

export const DeleteFaculty = (id: number) => call<boolean>('delete_faculty', { id })

export const RestoreFaculty = (id: number) => call<boolean>('restore_faculty', { id })

// ---------- Преподаватели ----------

export const CreateTeacher = (req: TeacherRequest) => call<Teacher>('create_teacher', { req })

export const GetTeachers = () => call<Teacher[]>('get_teachers')

export const GetTeachersByArchived = (isArchived: boolean) =>
  call<Teacher[]>('get_teachers_by_archived', { isArchived })

export const UpdateTeacher = (req: TeacherRequest) => call<boolean>('update_teacher', { req })

export const DeleteTeacher = (id: number) => call<boolean>('delete_teacher', { id })

export const RestoreTeacher = (id: number) => call<boolean>('restore_teacher', { id })

// ---------- Семестры ----------

export const CreateSemester = (req: SemesterRequest) => call<Semester>('create_semester', { req })

export const GetSemesters = () => call<Semester[]>('get_semesters')

export const GetSemestersByArchived = (isArchived: boolean) =>
  call<Semester[]>('get_semesters_by_archived', { isArchived })

export const UpdateSemester = (req: SemesterRequest) => call<boolean>('update_semester', { req })


export const DeleteSemester = (id: number) => call<boolean>('delete_semester', { id })

export const RestoreSemester = (id: number) => call<boolean>('restore_semester', { id })

export const GetAllTeachersForSemester = async (semesterId: number): Promise<ApiResponse<SemesterTeacher[]>> => {
  const resp = await call<Teacher[]>('get_all_teachers_for_semester', { semesterId })
  return {
    data: (resp.data ?? []).map(t => ({
      id: t.id,
      firstName: t.first_name,
      lastName: t.last_name,
      middleName: t.middle_name,
      directionId: t.direction_id,
      directionName: t.direction_name,
      rate: t.rate,
      isArchived: t.is_archived,
      isGuest: t.is_guest,
      isBound: t.is_bound
    })),
    error: resp.error
  }
}

export const BindTeacherToSemester = (req: BindRequest) =>
  call<boolean>('bind_teacher_to_semester', { req })

export const UnbindTeacherFromSemester = (req: BindRequest) =>
  call<boolean>('unbind_teacher_from_semester', { req })

// ---------- Расписание ----------

export const GetSchedule = (semesterId: number) =>
  call<ScheduleView>('get_schedule', { semesterId })

export const ToggleTeacherInSlot = (req: ToggleTeacherRequest) =>
  call<ScheduleOutcome>('toggle_teacher_in_slot', { req })

export const SetSlotFaculties = (req: SlotFacultiesRequest) =>
  call<boolean>('set_slot_faculties', { req })

export const AddSlot = (req: AddSlotRequest) => call<ScheduleOutcome>('add_slot', { req })

export const DeleteSlot = (req: DeleteSlotRequest) => call<boolean>('delete_slot', { req })

// ---------- Экспорт ----------

export const GetExportFileName = (semesterId: number) =>
  call<string>('get_export_file_name', { semesterId })

export const ExportScheduleXLSX = (semesterId: number, path: string) =>
  call<string>('export_schedule_xlsx', { semesterId, path })
