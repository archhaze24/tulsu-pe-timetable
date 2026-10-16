<script lang="ts">
  import { t } from 'svelte-i18n'
  import { navigate } from '../../stores/router'
  import { semestersStore, addSemester, updateSemester } from '../../stores/semesters'
  
  let { id = 0 }: { id?: number } = $props()

  const current = $derived($semestersStore.find(s => s.id === id))

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

  const canSave = $derived(Boolean(name.trim()) && Boolean(startDate) && Boolean(endDate) && endDate >= startDate)

  const back = () => { navigate('semesters') }

  async function save() {
    if (current) {
      const ok = await updateSemester(current.id, { name, startDate, endDate })
      if (ok) navigate('semesters')
    } else {
      const created = await addSemester({ name, startDate, endDate })
      if (created) navigate('schedule', { id: created.id })
    }
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t(current ? 'edit_semester' : 'new_semester')}</h2>

  <div class="grid gap-4">
    <div>
      <label class="field-label" for="sem-name">{$t('semester_name')}</label>
      <input id="sem-name" class="input mt-1" bind:value={name} placeholder={$t('semester_name_placeholder')} />
    </div>
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="field-label" for="sem-start">{$t('semester_start')}</label>
        <input id="sem-start" type="date" class="input mt-1" bind:value={startDate} />
      </div>
      <div>
        <label class="field-label" for="sem-end">{$t('semester_end')}</label>
        <input id="sem-end" type="date" class="input mt-1" bind:value={endDate} />
      </div>
    </div>
    <button class="btn-success mt-2 w-full rounded-xl px-4 py-3 text-base font-semibold shadow-md" onclick={save} disabled={!canSave}>
      {$t('save')}
    </button>
  </div>
</div>
