<script lang="ts">
  import { onMount } from 'svelte'
  import { t } from 'svelte-i18n'
  import HomeScreen from './lib/components/home/home-screen.svelte'
  import FacultiesScreen from './lib/components/faculties/faculties-screen.svelte'
  import FacultyEdit from './lib/components/faculties/faculty-edit.svelte'
  import FacultiesArchive from './lib/components/faculties/faculties-archive.svelte'
  import DirectionsScreen from './lib/components/directions/directions-screen.svelte'
  import DirectionEdit from './lib/components/directions/direction-edit.svelte'
  import DirectionsArchive from './lib/components/directions/directions-archive.svelte'
  import TeachersScreen from './lib/components/teachers/teachers-screen.svelte'
  import TeacherEdit from './lib/components/teachers/teacher-edit.svelte'
  import TeachersArchive from './lib/components/teachers/teachers-archive.svelte'
  import SemestersScreen from './lib/components/schedule/semesters-screen.svelte'
  import SemesterEdit from './lib/components/schedule/semester-edit.svelte'
  import SemestersArchive from './lib/components/schedule/semesters-archive.svelte'
  import ScheduleScreen from './lib/components/schedule/schedule-screen.svelte'
  import DebugMenu from './lib/components/ui/debug-menu.svelte'
  import Notices from './lib/components/ui/notices.svelte'
  import { route } from './lib/stores/router'
  import { GetConfig, UpdateConfig, type Config } from './lib/api'
  import { request } from './lib/stores/request'
  import { theme, setTheme, type ThemeMode } from './lib/stores/theme'
  import { debugMenuOpen, toggleDebugMenu, closeDebugMenu } from './lib/stores/debug'
  import { refreshDirections } from './lib/stores/directions'
  import { refreshFaculties } from './lib/stores/faculties'
  import { refreshTeachers } from './lib/stores/teachers'

  let configData = $state<Config | null>(null)
  const routeSemesterId = $derived(Number($route.params?.id ?? 0))

  $effect(() => {
    document.documentElement.classList.toggle('dark', $theme === 'dark')
  })

  async function loadConfig(): Promise<void> {
    const cfg = await request('GetConfig', () => GetConfig())
    if (!cfg) return
    configData = cfg
    setTheme(cfg.theme === 'dark' ? 'dark' : 'light')
  }

  async function onToggleTheme(): Promise<void> {
    // Без загруженного конфига нельзя сохранить: иначе dbPath перезапишется пустой строкой.
    if (!configData) return
    const prev = $theme
    const next: ThemeMode = prev === 'dark' ? 'light' : 'dark'
    setTheme(next)
    const cfg: Config = { ...configData, theme: next }
    if (await request('UpdateConfig', () => UpdateConfig(cfg))) {
      configData = cfg
    } else {
      setTheme(prev)
    }
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'F11') {
      e.preventDefault()
      toggleDebugMenu()
    }
  }

  onMount(() => {
    loadConfig()
    refreshDirections()
    refreshFaculties()
    refreshTeachers()
  })
</script>

<svelte:window onkeydown={onKeyDown} />

<main class="app-root {$route.name === 'schedule' ? '' : 'flex items-center justify-center'}">
  {#if $route.name === 'home'}
    <button class="theme-toggle"
      title={$theme === 'dark' ? $t('theme_light') : $t('theme_dark')} aria-label={$theme === 'dark' ? $t('theme_light') : $t('theme_dark')}
      disabled={!configData} onclick={onToggleTheme}>
      {$theme === 'dark' ? '☀️' : '🌙'}
    </button>
    <HomeScreen />
  {:else if $route.name === 'faculties'}
    <FacultiesScreen />
  {:else if $route.name === 'faculty_edit'}
    <FacultyEdit />
  {:else if $route.name === 'faculties_archive'}
    <FacultiesArchive />
  {:else if $route.name === 'directions'}
    <DirectionsScreen />
  {:else if $route.name === 'direction_edit'}
    <DirectionEdit />
  {:else if $route.name === 'directions_archive'}
    <DirectionsArchive />
  {:else if $route.name === 'teachers'}
    <TeachersScreen />
  {:else if $route.name === 'teacher_edit'}
    <TeacherEdit />
  {:else if $route.name === 'teachers_archive'}
    <TeachersArchive />
  {:else if $route.name === 'semesters'}
    <SemestersScreen />
  {:else if $route.name === 'semester_edit'}
    <SemesterEdit id={routeSemesterId} />
  {:else if $route.name === 'semesters_archive'}
    <SemestersArchive />
  {:else if $route.name === 'schedule'}
    <ScheduleScreen />
  {/if}

  <DebugMenu open={$debugMenuOpen} {configData} onclose={closeDebugMenu} />
  <Notices />
</main>
