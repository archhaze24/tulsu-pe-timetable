<script lang="ts">
  import HomeScreen from './lib/components/home/home-screen.svelte'
  import FacultiesScreen from './lib/components/faculties/faculties-screen.svelte'
  import FacultyEdit from './lib/components/faculties/faculty-edit.svelte'
  import DirectionsScreen from './lib/components/directions/directions-screen.svelte'
  import DirectionEdit from './lib/components/directions/direction-edit.svelte'
  import { route } from './lib/stores/router'
  import TeachersScreen from './lib/components/teachers/teachers-screen.svelte'
  import TeacherEdit from './lib/components/teachers/teacher-edit.svelte'
  import SemestersScreen from './lib/components/schedule/semesters-screen.svelte'
  import SemesterEdit from './lib/components/schedule/semester-edit.svelte'
  import ScheduleScreen from './lib/components/schedule/schedule-screen.svelte'
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

  // Вспомогательное значение для передачи id семестра в редактор
  $: routeSemesterId = Number(($route.params as any)?.id ?? 0)
</script>

<main class="min-h-screen bg-slate-900 text-slate-50 font-sans { $route.name === 'schedule' ? '' : 'flex items-center justify-center' }">
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
  {:else if $route.name === 'teachers'}
    <TeachersScreen />
  {:else if $route.name === 'teacher_edit'}
    <TeacherEdit />
  {:else if $route.name === 'semesters'}
    <SemestersScreen />
  {:else if $route.name === 'semester_edit'}
    <SemesterEdit id={routeSemesterId} />
  {:else if $route.name === 'schedule'}
    <ScheduleScreen />
  {/if}
</main>
