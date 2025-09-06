<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { navigate } from '../../stores/router'
  import { fetchArchivedSemesters, restoreSemester, type Semester } from '../../stores/semesters'

  const back = () => navigate('semesters')
  let archived: Semester[] = []

  async function load(): Promise<void> {
    archived = await fetchArchivedSemesters()
  }

  async function onRestore(id: number): Promise<void> {
    const ok = await restoreSemester(id)
    if (ok) archived = archived.filter(s => s.id !== id)
  }

  onMount(load)
</script>

<div class="max-w-2xl w-full px-6 py-10">
  <button class="mb-6 text-slate-500 hover:text-slate-700 dark:text-slate-300 dark:hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  <div class="flex items-center justify-between gap-3 mb-4">
    <h2 class="text-2xl font-semibold">{$t('semesters')} — {$t('archive')}</h2>
  </div>

  <div class="grid gap-3">
    {#each archived as s}
      <div class="flex items-center gap-3 rounded-lg bg-slate-100 ring-1 ring-black/5 dark:bg-slate-800/60 dark:ring-white/10 px-3 py-2">
        <div class="flex-1">
          <div class="font-medium">{s.name}</div>
          <div class="text-xs text-slate-600 dark:text-slate-300">{s.startDate} — {s.endDate}</div>
        </div>
        <button class="rounded-md bg-emerald-600/90 hover:bg-emerald-600 active:bg-emerald-700 px-3 py-2 text-xs font-medium transition text-white" on:click={() => onRestore(s.id)}>
          {$t('restore')}
        </button>
      </div>
    {/each}
    {#if archived.length === 0}
      <div class="text-sm text-slate-600 dark:text-slate-300">{$t('empty')}</div>
    {/if}
  </div>
</div>


