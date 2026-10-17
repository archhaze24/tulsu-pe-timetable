import { invoke } from '@tauri-apps/api/core'
import { get } from 'svelte/store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../i18n'
import type { ApiResponse, Teacher } from '../api'
import ru from '../locales/ru.json'
import { notices } from './notify'
import {
  bindTeacherToSemester,
  semesterTeachersStore,
  unbindTeacherFromSemester,
} from './semester-teachers'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const invokeMock = vi.mocked(invoke)

const SEMESTER_ID = 10

const teacher = (id: number): Teacher => ({
  id,
  firstName: 'Иван',
  lastName: 'Петров',
  middleName: 'Иванович',
  directionId: 1,
  rate: 1,
  directionName: 'Футбол',
  directionArchived: false,
  isArchived: false,
  isGuest: false,
  isBound: true,
})

function respond(responses: Record<string, ApiResponse<unknown>>) {
  invokeMock.mockImplementation(async (command: string) => {
    if (!(command in responses)) throw new Error(`Неожиданная команда: ${command}`)
    return responses[command]
  })
}

const calledCommands = () => invokeMock.mock.calls.map(([command]) => command)

beforeEach(() => {
  invokeMock.mockReset()
  semesterTeachersStore.set([])
  notices.set([])
  vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('bindTeacherToSemester', () => {
  it('возвращает true и перечитывает преподавателей семестра', async () => {
    respond({
      bind_teacher_to_semester: { data: true, error: '' },
      get_all_teachers_for_semester: { data: [teacher(1)], error: '' },
    })

    expect(await bindTeacherToSemester(SEMESTER_ID, 1)).toBe(true)
    expect(invokeMock).toHaveBeenCalledWith('bind_teacher_to_semester', {
      req: { semesterId: SEMESTER_ID, teacherId: 1 },
    })
    expect(invokeMock).toHaveBeenCalledWith('get_all_teachers_for_semester', { semesterId: SEMESTER_ID })
    expect(get(semesterTeachersStore).map(t => t.id)).toEqual([1])
  })

  it('при ошибке возвращает false, не меняет список и не перечитывает его', async () => {
    const existing = [teacher(2)]
    semesterTeachersStore.set(existing)
    respond({ bind_teacher_to_semester: { data: null, error: 'errors.semesters.bind_teacher_failed' } })

    expect(await bindTeacherToSemester(SEMESTER_ID, 1)).toBe(false)
    expect(get(semesterTeachersStore)).toEqual(existing)
    expect(calledCommands()).toEqual(['bind_teacher_to_semester'])
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.semesters.bind_teacher_failed }),
    ])
  })
})

describe('unbindTeacherFromSemester', () => {
  it('возвращает результат снятия и перечитывает преподавателей семестра', async () => {
    semesterTeachersStore.set([teacher(1)])
    respond({
      unbind_teacher_from_semester: { data: { assignments: 3, deletedPairs: 1 }, error: '' },
      get_all_teachers_for_semester: { data: [], error: '' },
    })

    expect(await unbindTeacherFromSemester(SEMESTER_ID, 1)).toEqual({ assignments: 3, deletedPairs: 1 })
    expect(invokeMock).toHaveBeenCalledWith('unbind_teacher_from_semester', {
      req: { semesterId: SEMESTER_ID, teacherId: 1 },
    })
    expect(get(semesterTeachersStore)).toEqual([])
  })

  it('при ошибке возвращает null, не меняет список и не перечитывает его', async () => {
    const existing = [teacher(1)]
    semesterTeachersStore.set(existing)
    respond({ unbind_teacher_from_semester: { data: null, error: 'errors.semesters.unbind_teacher_failed' } })

    expect(await unbindTeacherFromSemester(SEMESTER_ID, 1)).toBeNull()
    expect(get(semesterTeachersStore)).toEqual(existing)
    expect(calledCommands()).toEqual(['unbind_teacher_from_semester'])
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.semesters.unbind_teacher_failed }),
    ])
  })
})
