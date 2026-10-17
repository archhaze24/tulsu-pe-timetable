<script lang="ts">
  import { t } from 'svelte-i18n'
  import { navigate } from '../../stores/router'
  import { semestersStore, addSemester, updateSemester } from '../../stores/semesters'

  let { id = 0 }: { id?: number } = $props()

  const isEdit = $derived(id > 0)
  const current = $derived($semestersStore.find(s => s.id === id))
  const notFound = $derived(isEdit && !current)

  let name = $state('')
  let startDate = $state('')
  let endDate = $state('')
  let lastInitId: number | undefined

  $effect(() => {
    const sem = current
    if (sem && lastInitId !== sem.id) {
      name = sem.name
      startDate = sem.startDate
      endDate = sem.endDate
      lastInitId = sem.id
    }
  })

  const nameEmpty = $derived(!name.trim())
  const datesEmpty = $derived(!startDate || !endDate)
  const dateOrderInvalid = $derived(!datesEmpty && endDate < startDate)
  const canSave = $derived(!notFound && !nameEmpty && !datesEmpty && !dateOrderInvalid)

  const back = () => { navigate('semesters') }

  async function save() {
    if (!canSave) return
    if (isEdit) {
      if (await updateSemester({ id, name: name.trim(), startDate, endDate })) navigate('semesters')
    } else {
      const created = await addSemester({ name: name.trim(), startDate, endDate })
      if (created) navigate('schedule', { id: created.id })
    }
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t(isEdit ? 'edit_semester' : 'new_semester')}</h2>

  {#if notFound}
    <div class="mb-4 text-sm text-rose-600">{$t('errors.semesters.not_found')}</div>
  {/if}

  <div class="grid gap-4">
    <div>
      <label class="field-label" for="sem-name">{$t('semester_name')}</label>
      <input id="sem-name" class="input mt-1" bind:value={name} placeholder={$t('semester_name_placeholder')} disabled={notFound} />
      {#if nameEmpty && !notFound}
        <div class="mt-1 text-xs text-rose-600">{$t('validation.name_required')}</div>
      {/if}
    </div>
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="field-label" for="sem-start">{$t('semester_start')}</label>
        <input id="sem-start" type="date" class="input mt-1" bind:value={startDate} disabled={notFound} />
      </div>
      <div>
        <label class="field-label" for="sem-end">{$t('semester_end')}</label>
        <input id="sem-end" type="date" class="input mt-1" bind:value={endDate} disabled={notFound} />
      </div>
    </div>
    {#if datesEmpty && !notFound}
      <div class="-mt-2 text-xs text-rose-600">{$t('validation.dates_required')}</div>
    {:else if dateOrderInvalid}
      <div class="-mt-2 text-xs text-rose-600">{$t('errors.semesters.date_order')}</div>
    {/if}
    <button class="btn-success mt-2 w-full rounded-xl px-4 py-3 text-base font-semibold shadow-md" onclick={save} disabled={!canSave}>
      {$t('save')}
    </button>
  </div>
</div>
