import { invoke } from '@tauri-apps/api/core'
import { get } from 'svelte/store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../i18n'
import type { ApiResponse, Direction } from '../api'
import ru from '../locales/ru.json'
import { directionsStore, refreshDirections, saveDirection } from './directions'
import { notices } from './notify'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const invokeMock = vi.mocked(invoke)

const direction = (id: number, name = 'Футбол'): Direction => ({ id, name, isArchived: false })

function respond(responses: Record<string, ApiResponse<unknown>>) {
  invokeMock.mockImplementation(async (command: string) => {
    if (!(command in responses)) throw new Error(`Неожиданная команда: ${command}`)
    return responses[command]
  })
}

const calledCommands = () => invokeMock.mock.calls.map(([command]) => command)

beforeEach(() => {
  invokeMock.mockReset()
  directionsStore.set([])
  notices.set([])
  vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('refreshDirections', () => {
  it('при ошибке не очищает список и показывает сообщение', async () => {
    const existing = [direction(1)]
    directionsStore.set(existing)
    respond({ get_directions: { data: null, error: 'errors.directions.get_all_failed' } })

    expect(await refreshDirections()).toBe(false)
    expect(get(directionsStore)).toEqual(existing)
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.directions.get_all_failed }),
    ])
  })
})

describe('saveDirection', () => {
  const req = { name: 'Плавание', teacherIds: [1, 2] }

  it('возвращает true, сохраняет запрос и перечитывает направления и преподавателей', async () => {
    respond({
      save_direction: { data: direction(5, 'Плавание'), error: '' },
      get_directions: { data: [direction(5, 'Плавание')], error: '' },
      get_teachers: { data: [], error: '' },
    })

    expect(await saveDirection(req)).toBe(true)
    expect(invokeMock).toHaveBeenCalledWith('save_direction', { req })
    expect(calledCommands()).toEqual(expect.arrayContaining(['get_directions', 'get_teachers']))
    expect(get(directionsStore)).toEqual([direction(5, 'Плавание')])
  })

  it('при ошибке возвращает false, не меняет список и не перечитывает его', async () => {
    const existing = [direction(1)]
    directionsStore.set(existing)
    respond({ save_direction: { data: null, error: 'errors.directions.update_failed' } })

    expect(await saveDirection(req)).toBe(false)
    expect(get(directionsStore)).toEqual(existing)
    expect(calledCommands()).toEqual(['save_direction'])
    expect(get(notices)).toEqual([
      expect.objectContaining({ message: ru.errors.directions.update_failed }),
    ])
  })
})
