import { writable } from 'svelte/store'
import * as App from '../../../wailsjs/go/app_services/App.js'

export interface Direction {
  id: number
  name: string
}

export const directionsStore = writable<Direction[]>([])

function mapDirectionFromBackend(d: any): Direction {
  return {
    id: Number(d.id),
    name: String(d.name)
  }
}

export async function refreshDirections(): Promise<void> {
  try {
    const resp = await App.GetDirections()
    if (resp.error) {
      console.error('GetDirections error:', resp.error)
      return
    }
    const mapped = (resp.data ?? []).map(mapDirectionFromBackend)
    directionsStore.set(mapped)
  } catch (e) {
    console.error('GetDirections failed:', e)
  }
}

export async function fetchArchivedDirections(): Promise<Direction[]> {
  try {
    const resp = await App.GetDirectionsByArchived(true)
    if (resp.error) {
      console.error('GetDirectionsByArchived error:', resp.error)
      return []
    }
    return (resp.data ?? []).map(mapDirectionFromBackend)
  } catch (e) {
    console.error('GetDirectionsByArchived failed:', e)
    return []
  }
}

export async function restoreDirection(id: number): Promise<boolean> {
  try {
    const resp = await App.RestoreDirection(id)
    if (resp.error) {
      console.error('RestoreDirection error:', resp.error)
      return false
    }
    await refreshDirections()
    return Boolean(resp.data)
  } catch (e) {
    console.error('RestoreDirection failed:', e)
    return false
  }
}

export async function updateDirection(id: number, changes: { name: string }): Promise<void> {
  try {
    const resp = await App.UpdateDirection({ id, name: changes.name })
    if (resp.error) {
      console.error('UpdateDirection error:', resp.error)
      return
    }
    await refreshDirections()
  } catch (e) {
    console.error('UpdateDirection failed:', e)
  }
}

export async function createDirection(name: string): Promise<void> {
  try {
    const resp = await App.CreateDirection({ name })
    if (resp.error) {
      console.error('CreateDirection error:', resp.error)
      return
    }
    await refreshDirections()
  } catch (e) {
    console.error('CreateDirection failed:', e)
  }
}

export async function deleteDirection(id: number): Promise<void> {
  try {
    const resp = await App.DeleteDirection(id)
    if (resp.error) {
      console.error('DeleteDirection error:', resp.error)
      return
    }
    await refreshDirections()
  } catch (e) {
    console.error('DeleteDirection failed:', e)
  }
}

void refreshDirections()

