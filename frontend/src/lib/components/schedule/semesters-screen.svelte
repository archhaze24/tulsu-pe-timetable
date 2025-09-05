<script lang="ts">
  import { t } from 'svelte-i18n'
  import { navigate } from '../../stores/router'
  import { semestersStore, type Semester } from '../../stores/semesters'

  const back = () => navigate('home')
  const createNew = () => navigate('semester_edit', { id: 0 })
  const edit = (s: Semester) => navigate('semester_edit', { id: s.id })
  const open = (s: Semester) => navigate('schedule', { id: s.id })

  function formatRange(s: Semester): string {
    return `${s.startDate} — ${s.endDate}`
  }
</script>

<div class="max-w-2xl w-full px-6 py-10">
  <button class="mb-6 text-slate-500 hover:text-slate-700 dark:text-slate-300 dark:hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  <div class="flex items-center justify-between gap-3 mb-4">
    <h2 class="text-2xl font-semibold">{$t('semesters')}</h2>
    <button class="rounded-lg bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 text-white px-3 py-2 text-sm font-semibold transition" on:click={createNew}>
      {$t('new_semester')}
    </button>
  </div>

  <div class="grid gap-3">
    {#each $semestersStore as s}
      <div class="flex items-center gap-3 rounded-lg bg-slate-100 ring-1 ring-black/5 dark:bg-slate-800/60 dark:ring-white/10 px-3 py-2">
        <div class="flex-1">
          <div class="font-medium">{s.name}</div>
          <div class="text-xs text-slate-600 dark:text-slate-300">{formatRange(s)}</div>
        </div>
        <div class="flex items-center gap-2">
          <button class="rounded-md bg-slate-100 hover:bg-slate-200 active:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600 dark:active:bg-slate-800 ring-1 ring-black/5 dark:ring-white/10 px-3 py-2 text-xs font-medium" on:click={() => edit(s)}>
            {$t('edit')}
          </button>
          <button class="rounded-md bg-indigo-600/90 hover:bg-indigo-600 active:bg-indigo-700 text-white px-3 py-2 text-xs font-medium" on:click={() => open(s)}>
            {$t('open_schedule')}
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>


