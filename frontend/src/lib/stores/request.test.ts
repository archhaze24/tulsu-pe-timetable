import { get } from 'svelte/store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../i18n'
import type { ApiResponse } from '../api'
import ru from '../locales/ru.json'
import { notices } from './notify'
import { request, requestFlag, translateError } from './request'

const ok = <T>(data: T): ApiResponse<T> => ({ data, error: '' })
const fail = <T>(error: string): ApiResponse<T> => ({ data: null, error })

beforeEach(() => {
  vi.useFakeTimers()
  notices.set([])
  vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterEach(() => {
  vi.useRealTimers()
  vi.restoreAllMocks()
})

describe('request', () => {
  it('возвращает data при успехе и не показывает уведомлений', async () => {
    const result = await request('Test', async () => ok([1, 2]))

    expect(result).toEqual([1, 2])
    expect(get(notices)).toEqual([])
  })

  it('при resp.error возвращает null и показывает перевод кода ошибки', async () => {
    const result = await request('Test', async () => fail('errors.teachers.not_found'))

    expect(result).toBeNull()
    expect(get(notices)).toEqual([
      expect.objectContaining({ kind: 'error', message: ru.errors.teachers.not_found }),
    ])
  })

  it('при исключении в call возвращает null и показывает общую ошибку', async () => {
    const result = await request('Test', async () => {
      throw new Error('network down')
    })

    expect(result).toBeNull()
    expect(get(notices)).toEqual([
      expect.objectContaining({ kind: 'error', message: ru.errors.unknown }),
    ])
  })
})

describe('translateError', () => {
  it('возвращает перевод известного ключа', () => {
    expect(translateError('errors.teachers.not_found')).toBe(ru.errors.teachers.not_found)
  })

  it('для неизвестного ключа возвращает общий текст errors.unknown', () => {
    expect(translateError('errors.no_such_key')).toBe(ru.errors.unknown)
  })
})

describe('requestFlag', () => {
  it('возвращает true только при data === true', async () => {
    expect(await requestFlag('Test', async () => ok(true))).toBe(true)
    expect(get(notices)).toEqual([])
  })

  it('возвращает false при data === false без уведомления', async () => {
    expect(await requestFlag('Test', async () => ok(false))).toBe(false)
    expect(get(notices)).toEqual([])
  })

  it('возвращает false и показывает ошибку при resp.error', async () => {
    expect(await requestFlag('Test', async () => fail('errors.semesters.not_found'))).toBe(false)
    expect(get(notices)).toEqual([
      expect.objectContaining({ kind: 'error', message: ru.errors.semesters.not_found }),
    ])
  })

  it('возвращает false при исключении в call', async () => {
    const result = await requestFlag('Test', async () => {
      throw new Error('boom')
    })

    expect(result).toBe(false)
    expect(get(notices)).toEqual([
      expect.objectContaining({ kind: 'error', message: ru.errors.unknown }),
    ])
  })
})
