<script lang="ts">
  import { t } from 'svelte-i18n'
  import { teachersStore, addTeacher, updateTeacher } from '../../stores/teachers'
  import { directionsStore } from '../../stores/directions'
  import { navigate, route } from '../../stores/router'

  const teacherId = $derived(Number($route.params?.id ?? 0))
  const isEdit = $derived(teacherId > 0)
  const teacher = $derived($teachersStore.find(t => t.id === teacherId))
  const notFound = $derived(isEdit && !teacher)

  const STEP = 0.25

  let firstName = $state('')
  let lastName = $state('')
  let middleName = $state('')
  let directionId = $state(0)
  let rate = $state<number | null>(1)
  let lastInitId = 0

  // Ставка — положительное число с шагом 0,25 (Q-5): rate × 4 должно быть целым.
  const rateValid = $derived(rate !== null && Number.isFinite(rate) && rate > 0 && Math.abs(rate * 4 - Math.round(rate * 4)) < 1e-9)
  const firstNameEmpty = $derived(!firstName.trim())
  const lastNameEmpty = $derived(!lastName.trim())
  const canSave = $derived(!notFound && !firstNameEmpty && !lastNameEmpty && rateValid)

  $effect(() => {
    const cur = teacher
    if (cur && lastInitId !== cur.id) {
      firstName = cur.firstName
      lastName = cur.lastName
      middleName = cur.middleName
      directionId = cur.directionId
      rate = cur.rate
      lastInitId = cur.id
    }
    if (!cur) lastInitId = 0
  })

  const back = () => { navigate('teachers') }

  const save = async () => {
    if (!canSave || rate === null) return
    const req = {
      firstName: firstName.trim(),
      lastName: lastName.trim(),
      middleName: middleName.trim(),
      directionId: Number(directionId),
      rate
    }
    if (isEdit) {
      if (await updateTeacher({ ...req, id: teacherId })) back()
    } else if (await addTeacher(req)) {
      back()
    }
  }

  const incRate = () => {
    rate = Number(((rate ?? 0) + STEP).toFixed(2))
  }
  const decRate = () => {
    rate = Math.max(STEP, Number(((rate ?? 0) - STEP).toFixed(2)))
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t(isEdit ? 'edit_teacher' : 'create_teacher')}</h2>

  {#if notFound}
    <div class="mb-4 text-sm text-rose-600">{$t('errors.teachers.not_found')}</div>
  {/if}

  <label class="field-label mb-2 block" for="t-last">{$t('teacher_last_name')}</label>
  <input id="t-last" class="input" bind:value={lastName} placeholder={$t('teacher_last_name_placeholder')} disabled={notFound} />
  {#if lastNameEmpty && !notFound}
    <div class="mt-1 text-xs text-rose-600">{$t('validation.last_name_required')}</div>
  {/if}

  <label class="field-label mb-2 mt-4 block" for="t-first">{$t('teacher_first_name')}</label>
  <input id="t-first" class="input" bind:value={firstName} placeholder={$t('teacher_first_name_placeholder')} disabled={notFound} />
  {#if firstNameEmpty && !notFound}
    <div class="mt-1 text-xs text-rose-600">{$t('validation.first_name_required')}</div>
  {/if}

  <label class="field-label mb-2 mt-4 block" for="t-middle">{$t('teacher_middle_name')}</label>
  <input id="t-middle" class="input" bind:value={middleName} placeholder={$t('teacher_middle_name_placeholder')} disabled={notFound} />

  <label class="field-label mb-2 mt-4 block" for="t-direction">{$t('teacher_direction')}</label>
  <select id="t-direction" class="input appearance-none pr-10 focus:ring-2 focus:ring-emerald-500/40" bind:value={directionId} disabled={notFound}>
    <option value={0}>{$t('no_direction')}</option>
    {#each $directionsStore as dir (dir.id)}
      <option class="bg-white" value={dir.id}>{dir.name}</option>
    {/each}
    {#if teacher?.directionArchived}
      <option class="bg-white" value={teacher.directionId}>{teacher.directionName} {$t('archived_suffix')}</option>
    {/if}
  </select>

  <label class="field-label mb-2 mt-4 block" for="t-rate">{$t('teacher_rate')}</label>
  <div class="relative">
    <input id="t-rate" type="number" min="0" step="0.25" class="input appearance-none pr-12 focus:ring-2 focus:ring-emerald-500/40" bind:value={rate} placeholder="1.0" disabled={notFound} />
    <div class="absolute right-2 top-1/2 flex -translate-y-1/2 flex-col items-center gap-0">
      <button type="button" class="btn-neutral flex h-4 w-4 items-center justify-center rounded-md" aria-label={$t('increase')} onclick={incRate}>
        <svg class="h-2.5 w-2.5 text-slate-700" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path fill-rule="evenodd" d="M14.77 12.79a.75.75 0 01-1.06-.02L10 9.06l-3.71 3.71a.75.75 0 11-1.08-1.04l4.25-4.25a.75.75 0 011.08 0l4.25 4.25c.3.3.3.77 0 1.06z" clip-rule="evenodd" />
        </svg>
      </button>
      <button type="button" class="btn-neutral mt-0.5 flex h-4 w-4 items-center justify-center rounded-md" aria-label={$t('decrease')} onclick={decRate}>
        <svg class="h-2.5 w-2.5 text-slate-700" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
          <path fill-rule="evenodd" d="M14.77 7.21a.75.75 0 01-1.06.02L10 10.94 6.29 7.23a.75.75 0 10-1.08 1.04l4.25 4.25a.75.75 0 001.08 0l4.25-4.25a.75.75 0 00-.02-1.06z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
  </div>
  {#if !rateValid && !notFound}
    <div class="mt-1 text-xs text-rose-600">{$t('errors.teachers.rate_invalid')}</div>
  {/if}

  <div class="mt-6 flex gap-3">
    <button class="btn-success px-4 py-2 text-sm font-medium shadow-sm" onclick={save} disabled={!canSave}>
      {$t(isEdit ? 'save' : 'create')}
    </button>
    <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={back}>{$t('cancel')}</button>
  </div>
</div>
