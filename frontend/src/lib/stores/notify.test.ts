import { get } from 'svelte/store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { dismiss, notices, notify } from './notify'

beforeEach(() => {
  vi.useFakeTimers()
  notices.set([])
})

afterEach(() => {
  vi.useRealTimers()
})

describe('notify', () => {
  it('добавляет уведомление с видом error по умолчанию', () => {
    notify('Ошибка')

    expect(get(notices)).toEqual([{ id: expect.any(Number), kind: 'error', message: 'Ошибка' }])
  })

  it('принимает вид info', () => {
    notify('Готово', 'info')

    expect(get(notices)).toEqual([expect.objectContaining({ kind: 'info', message: 'Готово' })])
  })

  it('скрывает уведомление через 4 секунды', () => {
    notify('Ошибка')

    vi.advanceTimersByTime(3999)
    expect(get(notices)).toHaveLength(1)

    vi.advanceTimersByTime(1)
    expect(get(notices)).toHaveLength(0)
  })

  it('выдаёт уникальные id', () => {
    notify('первое')
    notify('второе')

    const [first, second] = get(notices)
    expect(first.id).not.toBe(second.id)
  })
})

describe('dismiss', () => {
  it('удаляет только уведомление с указанным id', () => {
    notify('первое')
    notify('второе')
    const [first, second] = get(notices)

    dismiss(first.id)

    expect(get(notices)).toEqual([second])
  })
})
