<script lang="ts">
  import { t } from 'svelte-i18n'
  import { teachersStore, type Teacher, addTeacher, updateTeacher, formatTeacherName } from '../../stores/teachers'
  import { directionsStore, type Direction } from '../../stores/directions'
  import { navigate, route } from '../../stores/router'
  import { derived } from 'svelte/store'

  const back = () => navigate('teachers')
  const teacherId = derived(route, ($route) => Number($route.params?.id ?? 0))
  const teacher = derived([teachersStore, teacherId], ([$teachers, $id]) => $teachers.find(t => t.id === $id))

  let firstName = ''
  let lastName = ''
  let middleName = ''
  let directionId: number = 0
  let rate: number = 1

  $: if ($teacher && firstName === '' && lastName === '' && directionId === 0) {
    firstName = $teacher.firstName
    lastName = $teacher.lastName
    middleName = $teacher.middleName ?? ''
    directionId = $teacher.directionId
    rate = $teacher.rate
  }

  const save = () => {
    const data = {
      firstName: firstName.trim(),
      lastName: lastName.trim(),
      middleName: middleName.trim() || undefined,
      directionId: Number(directionId),
      rate: Number(rate)
    }
    if (!data.firstName || !data.lastName || !data.directionId || !data.rate) return
    if ($teacher) {
      updateTeacher($teacher.id, data)
    } else {
      addTeacher(data)
    }
    back()
  }

  const step = 0.25
  const clamp = (value: number) => Math.max(0, value)
  const incRate = () => {
    rate = Number((rate + step).toFixed(2))
  }
  const decRate = () => {
    rate = clamp(Number((rate - step).toFixed(2)))
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-300 hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  {#if $teacher}
    <h2 class="text-2xl font-semibold mb-4">{$t('edit_teacher')}</h2>
  {:else}
    <h2 class="text-2xl font-semibold mb-4">{$t('create_teacher')}</h2>
  {/if}

  <label class="block text-sm mb-2 text-slate-300" for="t-last">{$t('teacher_last_name')}</label>
  <input id="t-last" class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={lastName} placeholder={$t('teacher_last_name_placeholder')} />

  <label class="block text-sm mb-2 mt-4 text-slate-300" for="t-first">{$t('teacher_first_name')}</label>
  <input id="t-first" class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={firstName} placeholder={$t('teacher_first_name_placeholder')} />

  <label class="block text-sm mb-2 mt-4 text-slate-300" for="t-middle">{$t('teacher_middle_name')}</label>
  <input id="t-middle" class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={middleName} placeholder={$t('teacher_middle_name_placeholder')} />

  <label class="block text-sm mb-2 mt-4 text-slate-300" for="t-direction">{$t('teacher_direction')}</label>
  <div class="relative">
    <select id="t-direction" class="w-full appearance-none rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 pr-10 py-2 ring-1 ring-white/10 focus:ring-2 focus:ring-emerald-500/40 outline-none text-slate-100" bind:value={directionId}>
      <option value="0" disabled>{$t('select_direction')}</option>
      {#each $directionsStore as dir}
        <option class="bg-slate-900 text-slate-100" value={dir.id}>{dir.name}</option>
      {/each}
    </select>
    <svg class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-300" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
      <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 10.94l3.71-3.71a.75.75 0 111.08 1.04l-4.25 4.25a.75.75 0 01-1.08 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
    </svg>
  </div>

  <label class="block text-sm mb-2 mt-4 text-slate-300" for="t-rate">{$t('teacher_rate')}</label>
  <div class="relative">
    <input id="t-rate" type="number" min="0" step="0.25" class="w-full appearance-none rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 pr-12 py-2 ring-1 ring-white/10 focus:ring-2 focus:ring-emerald-500/40 outline-none" bind:value={rate} placeholder="1.0" />
    <div class="absolute right-2 top-1/2 -translate-y-1/2 flex flex-col items-center gap-0">
      <button type="button" class="h-4 w-4 rounded-md bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 ring-1 ring-white/10 flex items-center justify-center" aria-label={$t('increase')} on:click={incRate}>
        <svg class="h-2.5 w-2.5 text-slate-200" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path fill-rule="evenodd" d="M14.77 12.79a.75.75 0 01-1.06-.02L10 9.06l-3.71 3.71a.75.75 0 11-1.08-1.04l4.25-4.25a.75.75 0 011.08 0l4.25 4.25c.3.3.3.77 0 1.06z" clip-rule="evenodd" />
        </svg>
      </button>
      <button type="button" class="h-4 w-4 rounded-md bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 ring-1 ring-white/10 flex items-center justify-center mt-0.5" aria-label={$t('decrease')} on:click={decRate}>
        <svg class="h-2.5 w-2.5 text-slate-200" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 10.94l3.71-3.71a.75.75 0 111.08 1.04l-4.25 4.25a.75.75 0 01-1.08 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
  </div>

  <div class="mt-6 flex gap-3">
    <button class="rounded-lg bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 px-4 py-2 text-sm font-medium shadow-sm transition" on:click={save}>
      {$t($teacher ? 'save' : 'create')}
    </button>
    <button class="rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-white/10 transition" on:click={back}>
      {$t('cancel')}
    </button>
  </div>
</div>


