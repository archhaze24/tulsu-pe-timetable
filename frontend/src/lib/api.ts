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
import type { TeacherView } from './generated/TeacherView'
import type { TimePair } from './generated/TimePair'
import type { UnbindResult } from './generated/UnbindResult'

import type { SaveDirectionRequest } from './generated/SaveDirectionRequest'
import type { NameRequest } from './generated/NameRequest'
import type { UpdateNameRequest } from './generated/UpdateNameRequest'
import type { TeacherRequest } from './generated/TeacherRequest'
import type { SemesterRequest } from './generated/SemesterRequest'
import type { BindRequest } from './generated/BindRequest'
import type { ToggleTeacherRequest } from './generated/ToggleTeacherRequest'
import type { SlotFacultiesRequest } from './generated/SlotFacultiesRequest'
import type { AddSlotRequest } from './generated/AddSlotRequest'
import type { SetSlotTeacherCountRequest } from './generated/SetSlotTeacherCountRequest'
import type { SlotRequest } from './generated/SlotRequest'

export type { Config, Direction, Faculty, Teacher, TeacherRequest, Semester, SemesterRequest, SaveDirectionRequest }
export type { ScheduleOutcome, ScheduleView, TeacherView, TimePair, UnbindResult }
export type { SlotView } from './generated/SlotView'
export type { DayView } from './generated/DayView'

// `data` равен `null`, если `error` не пустой.
export interface ApiResponse<T> {
  data: T | null
  error: string
}

const call = <T>(command: string, args?: Record<string, unknown>): Promise<ApiResponse<T>> =>
  invoke<ApiResponse<T>>(command, args)

// ---------- Конфиг ----------

export const GetConfig = () => call<Config>('get_config')


// ---------- Направления ----------

export const GetDirections = () => call<Direction[]>('get_directions')

export const GetDirectionsByArchived = (isArchived: boolean) =>
  call<Direction[]>('get_directions_by_archived', { isArchived })

export const SaveDirection = (req: SaveDirectionRequest) => call<Direction>('save_direction', { req })

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

export const GetAllTeachersForSemester = (semesterId: number) =>
  call<Teacher[]>('get_all_teachers_for_semester', { semesterId })

export const BindTeacherToSemester = (req: BindRequest) =>
  call<boolean>('bind_teacher_to_semester', { req })

export const PreviewUnbindTeacher = (req: BindRequest) =>
  call<UnbindResult>('preview_unbind_teacher', { req })

export const UnbindTeacherFromSemester = (req: BindRequest) =>
  call<UnbindResult>('unbind_teacher_from_semester', { req })

// ---------- Расписание ----------

export const GetSchedule = (semesterId: number) =>
  call<ScheduleView>('get_schedule', { semesterId })

export const ToggleTeacherInSlot = (req: ToggleTeacherRequest) =>
  call<ScheduleOutcome>('toggle_teacher_in_slot', { req })

export const SetSlotFaculties = (req: SlotFacultiesRequest) =>
  call<boolean>('set_slot_faculties', { req })

export const AddSlot = (req: AddSlotRequest) => call<ScheduleOutcome>('add_slot', { req })

export const SetSlotTeacherCount = (req: SetSlotTeacherCountRequest) =>
  call<ScheduleOutcome>('set_slot_teacher_count', { req })

export const ClearSlotAssignments = (req: SlotRequest) => call<boolean>('clear_slot_assignments', { req })

export const DeleteSlot = (req: SlotRequest) => call<boolean>('delete_slot', { req })

// ---------- Экспорт ----------

export const GetExportFileName = (semesterId: number) =>
  call<string>('get_export_file_name', { semesterId })

export const ExportScheduleXLSX = (semesterId: number, path: string) =>
  call<string>('export_schedule_xlsx', { semesterId, path })
