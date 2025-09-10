<script lang="ts">
  import { t } from 'svelte-i18n'
  import { navigate } from '../../stores/router'
  import { semestersStore, addSemester, updateSemester, type Semester } from '../../stores/semesters'

  export let id: number | undefined

  let name = ''
  let startDate = ''
  let endDate = ''
  let lastInitId: number | undefined = undefined

  $: canSave = Boolean(name.trim()) && Boolean(startDate) && Boolean(endDate) && (endDate >= startDate)

  $: current = $semestersStore.find(s => s.id === id)
  $: if (current && lastInitId !== current.id) {
    name = current.name
    // Ensure date inputs receive only YYYY-MM-DD
    startDate = (current.startDate || '').split('T')[0].split(' ')[0]
    endDate = (current.endDate || '').split('T')[0].split(' ')[0]
    lastInitId = current.id
  }

  const back = () => navigate('semesters')

  async function save() {
    if (current) {
      const updated = await updateSemester(current.id, { name, startDate, endDate })
      if (updated) navigate('semesters')
    } else {
      const created = await addSemester({ name, startDate, endDate })
      if (created) navigate('schedule', { id: created.id })
    }
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-500 hover:text-slate-700 dark:text-slate-300 dark:hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  <h2 class="text-2xl font-semibold mb-4">{$t(current ? 'edit_semester' : 'new_semester')}</h2>

  <div class="grid gap-4">
    <div>
      <label class="text-sm text-slate-600 dark:text-slate-300" for="sem-name">{$t('semester_name')}</label>
      <input id="sem-name" class="mt-1 w-full rounded-lg bg-white dark:bg-slate-900/60 focus:bg-white dark:focus:bg-slate-900 px-4 py-2 ring-1 ring-slate-200 dark:ring-white/10 outline-none text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-400" bind:value={name} placeholder={$t('semester_name_placeholder')} />
    </div>
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="text-sm text-slate-600 dark:text-slate-300" for="sem-start">{$t('semester_start')}</label>
        <input id="sem-start" type="date" class="mt-1 w-full rounded-lg bg-white dark:bg-slate-900/60 focus:bg-white dark:focus:bg-slate-900 px-4 py-2 ring-1 ring-slate-200 dark:ring-white/10 outline-none text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-400" bind:value={startDate} />
      </div>
      <div>
        <label class="text-sm text-slate-600 dark:text-slate-300" for="sem-end">{$t('semester_end')}</label>
        <input id="sem-end" type="date" class="mt-1 w-full rounded-lg bg-white dark:bg-slate-900/60 focus:bg-white dark:focus:bg-slate-900 px-4 py-2 ring-1 ring-slate-200 dark:ring-white/10 outline-none text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-400" bind:value={endDate} />
      </div>
    </div>
    <button class="mt-2 w-full rounded-xl bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed text-white px-4 py-3 text-base font-semibold shadow-md transition" on:click={save} disabled={!canSave}>
      {$t('save')}
    </button>
  </div>
</div>


