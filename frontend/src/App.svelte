<script lang="ts">
  import HomeScreen from './lib/components/home/home-screen.svelte'
  import FacultiesScreen from './lib/components/faculties/faculties-screen.svelte'
  import FacultyEdit from './lib/components/faculties/faculty-edit.svelte'
  import DirectionsScreen from './lib/components/directions/directions-screen.svelte'
  import DirectionEdit from './lib/components/directions/direction-edit.svelte'
  import { route } from './lib/stores/router'
  import {GetConfig, GetConfigPath} from '../wailsjs/go/app_services/App.js'
  import type {config} from '../wailsjs/go/models'

  // Инициализация конфигурации при старте приложения
  let configData: config.Config | null = null

  async function loadConfig(): Promise<void> {
    try {
      const configResponse = await GetConfig()
      if (!configResponse.error) {
        configData = configResponse.data
        console.log('Конфигурация загружена:', configData)
      } else {
        console.error('Ошибка загрузки конфигурации:', configResponse.error)
      }
    } catch (error) {
      console.error('Ошибка инициализации конфигурации:', error)
    }
  }

  // Загружаем конфигурацию при старте
  loadConfig()
</script>

<main class="min-h-screen bg-slate-900 text-slate-50 font-sans flex items-center justify-center">
  {#if $route.name === 'home'}
    <HomeScreen {configData} />
  {:else if $route.name === 'faculties'}
    <FacultiesScreen />
  {:else if $route.name === 'faculty_edit'}
    <FacultyEdit />
  {:else if $route.name === 'directions'}
    <DirectionsScreen />
  {:else if $route.name === 'direction_edit'}
    <DirectionEdit />
  {/if}
</main>
