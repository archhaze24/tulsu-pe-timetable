<script lang="ts">
  import { t } from 'svelte-i18n'
  import { derived } from 'svelte/store'
  import { teachersStore, addTeacher, updateTeacher } from '../../stores/teachers'
  import { directionsStore } from '../../stores/directions'
  import { navigate, route } from '../../stores/router'
  
  const teacherId = derived(route, ($route) => Number($route.params?.id ?? 0))
  const teacher = derived([teachersStore, teacherId], ([$teachers, $id]) => $teachers.find(t => t.id === $id))

  const STEP = 0.25

  let firstName = $state('')
  let lastName = $state('')
  let middleName = $state('')
  let directionId = $state(0)
  let rate = $state(1)
  let lastInitId = 0

  const canSave = $derived(Boolean(firstName.trim()) && Boolean(lastName.trim()) && Number.isFinite(Number(rate)))

  $effect(() => {
    const cur = $teacher
    if (cur && lastInitId !== cur.id) {
      firstName = cur.firstName
      lastName = cur.lastName
      middleName = cur.middleName ?? ''
      directionId = cur.directionId
      rate = cur.rate
      lastInitId = cur.id
    }
    if (!cur) lastInitId = 0
  })

  const back = () => { navigate('teachers') }

  const save = async () => {
    const data = {
      firstName: firstName.trim(),
      lastName: lastName.trim(),
      middleName: middleName.trim() || undefined,
      directionId: Number(directionId),
      rate: Number(rate)
    }
    if (!data.firstName || !data.lastName || !data.rate) return
    if ($teacher) {
      await updateTeacher($teacher.id, data)
    } else {
      await addTeacher(data)
    }
    back()
  }

  const incRate = () => {
    rate = Number((Number(rate) + STEP).toFixed(2))
  }
  const decRate = () => {
    rate = Math.max(0, Number((Number(rate) - STEP).toFixed(2)))
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t($teacher ? 'edit_teacher' : 'create_teacher')}</h2>

  <label class="field-label mb-2 block" for="t-last">{$t('teacher_last_name')}</label>
  <input id="t-last" class="input" bind:value={lastName} placeholder={$t('teacher_last_name_placeholder')} />

  <label class="field-label mb-2 mt-4 block" for="t-first">{$t('teacher_first_name')}</label>
  <input id="t-first" class="input" bind:value={firstName} placeholder={$t('teacher_first_name_placeholder')} />

  <label class="field-label mb-2 mt-4 block" for="t-middle">{$t('teacher_middle_name')}</label>
  <input id="t-middle" class="input" bind:value={middleName} placeholder={$t('teacher_middle_name_placeholder')} />

  <label class="field-label mb-2 mt-4 block" for="t-direction">{$t('teacher_direction')}</label>
  <select id="t-direction" class="input appearance-none pr-10 focus:ring-2 focus:ring-emerald-500/40" bind:value={directionId}>
    <option value={0}>{$t('no_direction')}</option>
    {#each $directionsStore as dir (dir.id)}
      <option class="bg-white dark:bg-slate-900 dark:text-slate-100" value={dir.id}>{dir.name}</option>
    {/each}
  </select>

  <label class="field-label mb-2 mt-4 block" for="t-rate">{$t('teacher_rate')}</label>
  <div class="relative">
    <input id="t-rate" type="number" min="0" step="0.25" class="input appearance-none pr-12 focus:ring-2 focus:ring-emerald-500/40" bind:value={rate} placeholder="1.0" />
    <div class="absolute right-2 top-1/2 flex -translate-y-1/2 flex-col items-center gap-0">
      <button type="button" class="btn-neutral flex h-4 w-4 items-center justify-center rounded-md" aria-label={$t('increase')} onclick={incRate}>
        <svg class="h-2.5 w-2.5 text-slate-700 dark:text-slate-200" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path fill-rule="evenodd" d="M14.77 12.79a.75.75 0 01-1.06-.02L10 9.06l-3.71 3.71a.75.75 0 11-1.08-1.04l4.25-4.25a.75.75 0 011.08 0l4.25 4.25c.3.3.3.77 0 1.06z" clip-rule="evenodd" />
        </svg>
      </button>
      <button type="button" class="btn-neutral mt-0.5 flex h-4 w-4 items-center justify-center rounded-md" aria-label={$t('decrease')} onclick={decRate}>
        <svg class="h-2.5 w-2.5 text-slate-700 dark:text-slate-200" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path fill-rule="evenodd" d="M14.77 7.21a.75.75 0 01-1.06.02L10 10.94 6.29 7.23a.75.75 0 10-1.08 1.04l4.25 4.25a.75.75 0 001.08 0l4.25-4.25a.75.75 0 00-.02-1.06z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
  </div>

  <div class="mt-6 flex gap-3">
    <button class="btn-success px-4 py-2 text-sm font-medium shadow-sm" onclick={save} disabled={!canSave}>
      {$t($teacher ? 'save' : 'create')}
    </button>
    <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={back}>{$t('cancel')}</button>
  </div>
</div>
