import { invoke } from '@tauri-apps/api/core'
import { get } from 'svelte/store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../i18n'
import type { ApiResponse, Teacher } from '../api'
import ru from '../locales/ru.json'
import { notices } from './notify'
import { addTeacher, formatTeacherName, refreshTeachers, teachersStore, updateTeacher } from './teachers'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const invokeMock = vi.mocked(invoke)

const teacher = (id: number, lastName = 'Петров'): Teacher => ({
  id,
  firstName: 'Иван',
  lastName,
  middleName: 'Иванович',
  directionId: 1,
  rate: 1,
  directionName: 'Футбол',
  directionArchived: false,
  isArchived: false,
  isGuest: false,
  isBound: false,
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
  teachersStore.set([])
  notices.set([])
  vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('refreshTeachers', () => {
  it('заменяет список данными с бэкенда', async () => {
    respond({ get_teachers: { data: [teacher(1), teacher(2)], error: '' } })

    expect(await refreshTeachers()).toBe(true)
    expect(get(teachersStore).map(t => t.id)).toEqual([1, 2])
  })

  it('при ошибке не очищает список и показывает сообщение', async () => {
    const existing = [teacher(1)]
    teachersStore.set(existing)
    respond({ get_teachers: { data: null, error: 'errors.teachers.get_all_failed' } })

    expect(await refreshTeachers()).toBe(false)
    expect(get(teachersStore)).toEqual(existing)
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.teachers.get_all_failed }),
    ])
  })
})

describe('addTeacher', () => {
  const req = { firstName: 'Анна', lastName: 'Смирнова', middleName: 'Игоревна', directionId: 0, rate: 1 }

  it('возвращает созданного преподавателя и перечитывает список', async () => {
    const created = teacher(3, 'Смирнова')
    respond({
      create_teacher: { data: created, error: '' },
      get_teachers: { data: [teacher(1), created], error: '' },
    })

    expect(await addTeacher(req)).toEqual(created)
    expect(invokeMock).toHaveBeenCalledWith('create_teacher', { req })
    expect(get(teachersStore).map(t => t.id)).toEqual([1, 3])
  })

  it('при ошибке возвращает null, не меняет список и не перечитывает его', async () => {
    const existing = [teacher(1)]
    teachersStore.set(existing)
    respond({ create_teacher: { data: null, error: 'errors.teachers.create_failed' } })

    expect(await addTeacher(req)).toBeNull()
    expect(get(teachersStore)).toEqual(existing)
    expect(calledCommands()).toEqual(['create_teacher'])
  })
})

describe('updateTeacher', () => {
  const req = { id: 1, firstName: 'Иван', lastName: 'Петров-Водкин', middleName: 'Иванович', directionId: 1, rate: 0.5 }

  it('возвращает true и перечитывает список при успехе', async () => {
    respond({
      update_teacher: { data: true, error: '' },
      get_teachers: { data: [{ ...teacher(1), lastName: 'Петров-Водкин', rate: 0.5 }], error: '' },
    })

    expect(await updateTeacher(req)).toBe(true)
    expect(get(teachersStore)[0].lastName).toBe('Петров-Водкин')
  })

  it('возвращает false при data === false без перечитывания', async () => {
    respond({ update_teacher: { data: false, error: '' } })

    expect(await updateTeacher(req)).toBe(false)
    expect(calledCommands()).toEqual(['update_teacher'])
  })

  it('возвращает false и показывает ошибку при resp.error', async () => {
    respond({ update_teacher: { data: null, error: 'errors.teachers.rate_invalid' } })

    expect(await updateTeacher(req)).toBe(false)
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.teachers.rate_invalid }),
    ])
  })
})

describe('formatTeacherName', () => {
  it('выводит фамилию и инициалы имени и отчества', () => {
    expect(formatTeacherName({ firstName: 'Иван', middleName: 'Петрович', lastName: 'Иванов' })).toBe('Иванов И. П.')
  })

  it('пропускает пустое отчество', () => {
    expect(formatTeacherName({ firstName: 'Иван', middleName: '', lastName: 'Иванов' })).toBe('Иванов И.')
  })

  it('без имени и отчества возвращает только фамилию', () => {
    expect(formatTeacherName({ firstName: '', middleName: '', lastName: 'Иванов' })).toBe('Иванов')
  })
})
