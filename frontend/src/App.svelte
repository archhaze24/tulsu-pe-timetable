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
  import {GetConfig} from '../wailsjs/go/app_services/App.js'
  import type {config} from '../wailsjs/go/models'
  import { theme, setTheme, toggleTheme } from './lib/stores/theme'
  import { UpdateConfig } from '../wailsjs/go/app_services/App.js'

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

  // Применяем тему: инициализируем стор из конфига и следим за изменениями
  $: if (configData?.theme) setTheme(configData.theme as any)
  $: {
    const htmlEl = typeof document !== 'undefined' ? document.documentElement : null
    if (!htmlEl) {}
    else {
      $theme === 'dark' ? htmlEl.classList.add('dark') : htmlEl.classList.remove('dark')
    }
  }

  async function onToggleTheme(): Promise<void> {
    const next = $theme === 'dark' ? 'light' : 'dark'
    setTheme(next as any)
    // Сохраняем в конфиг
    const cfg: config.Config = { dbPath: configData?.dbPath || '', theme: next }
    configData = cfg
    const res = await UpdateConfig(cfg)
    if (res && res.error) {
      // откат в случае ошибки
      const prev = next === 'dark' ? 'light' : 'dark'
      setTheme(prev as any)
      configData = { dbPath: cfg.dbPath, theme: prev } as config.Config
      console.error('Ошибка сохранения темы:', res.error)
    }
  }
</script>

<main class="min-h-screen bg-white text-slate-900 dark:bg-slate-900 dark:text-slate-50 font-sans { $route.name === 'schedule' ? '' : 'flex items-center justify-center' }">
  {#if $route.name === 'home'}
    <!-- Кнопка переключения темы только на главной -->
    <button class="fixed top-2 right-3 z-50 rounded-md px-3 py-1.5 text-base transition hover:opacity-80 bg-white/70 dark:bg-slate-800/60 ring-1 ring-black/10 dark:ring-white/10 backdrop-blur"
      title={$theme === 'dark' ? 'Светлая тема' : 'Тёмная тема'} aria-label={$theme === 'dark' ? 'Светлая тема' : 'Тёмная тема'} on:click={onToggleTheme}>
      {#if $theme === 'dark'}
        ☀️
      {:else}
        🌙
      {/if}
    </button>
  {/if}
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
