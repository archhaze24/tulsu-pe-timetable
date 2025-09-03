<script lang="ts">
  import { t } from 'svelte-i18n'
  import { navigate } from '../../stores/router'
  import { semestersStore, addSemester, updateSemester, type Semester } from '../../stores/semesters'

  export let id: number | undefined

  let name = ''
  let startDate = ''
  let endDate = ''

  $: current = $semestersStore.find(s => s.id === id)
  $: {
    if (current) {
      name = current.name
      startDate = current.startDate
      endDate = current.endDate
    }
  }

  const back = () => navigate('semesters')

  function save() {
    if (current) {
      updateSemester(current.id, { name, startDate, endDate })
      navigate('semesters')
    } else {
      const created = addSemester({ name, startDate, endDate })
      navigate('schedule', { id: created.id })
    }
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-300 hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  <h2 class="text-2xl font-semibold mb-4">{$t(current ? 'edit_semester' : 'new_semester')}</h2>

  <div class="grid gap-4">
    <div>
      <label class="text-sm text-slate-300" for="sem-name">{$t('semester_name')}</label>
      <input id="sem-name" class="mt-1 w-full rounded-md bg-slate-800 px-3 py-2 ring-1 ring-white/10 focus:outline-none focus:ring-2 focus:ring-indigo-500" bind:value={name} placeholder={$t('semester_name_placeholder')} />
    </div>
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="text-sm text-slate-300" for="sem-start">{$t('semester_start')}</label>
        <input id="sem-start" type="date" class="mt-1 w-full rounded-md bg-slate-800 px-3 py-2 ring-1 ring-white/10 focus:outline-none focus:ring-2 focus:ring-indigo-500" bind:value={startDate} />
      </div>
      <div>
        <label class="text-sm text-slate-300" for="sem-end">{$t('semester_end')}</label>
        <input id="sem-end" type="date" class="mt-1 w-full rounded-md bg-slate-800 px-3 py-2 ring-1 ring-white/10 focus:outline-none focus:ring-2 focus:ring-indigo-500" bind:value={endDate} />
      </div>
    </div>
    <button class="mt-2 w-full rounded-xl bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 px-4 py-3 text-base font-semibold shadow-md transition" on:click={save}>
      {$t('save')}
    </button>
  </div>
</div>


