import { invoke } from '@tauri-apps/api/core'
import { get } from 'svelte/store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../i18n'
import type { ApiResponse, Semester } from '../api'
import ru from '../locales/ru.json'
import { notices } from './notify'
import { addSemester, loadSemesters, semestersStore } from './semesters'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const invokeMock = vi.mocked(invoke)

const semester = (id: number, startDate: string): Semester => ({
  id,
  name: `Семестр ${id}`,
  startDate,
  endDate: '2099-01-31',
  isArchived: false,
})

// Ответы команд Tauri по имени команды; неожиданная команда роняет тест.
function respond(responses: Record<string, ApiResponse<unknown>>) {
  invokeMock.mockImplementation(async (command: string) => {
    if (!(command in responses)) throw new Error(`Неожиданная команда: ${command}`)
    return responses[command]
  })
}

const calledCommands = () => invokeMock.mock.calls.map(([command]) => command)

beforeEach(() => {
  invokeMock.mockReset()
  semestersStore.set([])
  notices.set([])
  vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('loadSemesters', () => {
  it('сортирует семестры по дате начала по убыванию', async () => {
    respond({
      get_semesters: {
        data: [semester(1, '2025-02-01'), semester(2, '2026-09-01'), semester(3, '2025-09-01')],
        error: '',
      },
    })

    expect(await loadSemesters()).toBe(true)
    expect(get(semestersStore).map(s => s.id)).toEqual([2, 3, 1])
  })

  it('при ошибке не очищает список и показывает сообщение', async () => {
    const existing = [semester(1, '2025-02-01')]
    semestersStore.set(existing)
    respond({ get_semesters: { data: null, error: 'errors.semesters.get_all_failed' } })

    expect(await loadSemesters()).toBe(false)
    expect(get(semestersStore)).toEqual(existing)
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.semesters.get_all_failed }),
    ])
  })
})

describe('addSemester', () => {
  const req = { name: 'Осенний 2026', startDate: '2026-09-01', endDate: '2027-01-31' }

  it('возвращает созданный семестр и перечитывает список', async () => {
    const created = semester(4, '2026-09-01')
    respond({
      create_semester: { data: created, error: '' },
      get_semesters: { data: [created, semester(1, '2025-02-01')], error: '' },
    })

    expect(await addSemester(req)).toEqual(created)
    expect(invokeMock).toHaveBeenCalledWith('create_semester', { req })
    expect(get(semestersStore).map(s => s.id)).toEqual([4, 1])
  })

  it('при ошибке возвращает null, не меняет список и не перечитывает его', async () => {
    const existing = [semester(1, '2025-02-01')]
    semestersStore.set(existing)
    respond({ create_semester: { data: null, error: 'errors.semesters.create_failed' } })

    expect(await addSemester(req)).toBeNull()
    expect(get(semestersStore)).toEqual(existing)
    expect(calledCommands()).toEqual(['create_semester'])
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.semesters.create_failed }),
    ])
  })
})
