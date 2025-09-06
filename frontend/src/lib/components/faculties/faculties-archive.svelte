<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { navigate } from '../../stores/router'
  import { fetchArchivedFaculties, restoreFaculty, type Faculty } from '../../stores/faculties'

  const back = () => navigate('faculties')
  let archived: Faculty[] = []

  async function load(): Promise<void> {
    archived = await fetchArchivedFaculties()
  }

  async function onRestore(id: number): Promise<void> {
    const ok = await restoreFaculty(id)
    if (ok) archived = archived.filter(f => f.id !== id)
  }

  onMount(load)
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-500 hover:text-slate-700 dark:text-slate-300 dark:hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  <div class="flex items-center justify-between gap-3 mb-4">
    <h2 class="text-2xl font-semibold">{$t('faculties')} — {$t('archive')}</h2>
  </div>

  <div class="mt-4 grid grid-cols-1 gap-3">
    {#each archived as faculty}
      <div class="w-full rounded-lg bg-slate-100 px-2 py-2 ring-1 ring-black/5 dark:bg-slate-800/60 dark:ring-white/10 shadow-sm flex items-center gap-2">
        <div class="flex-1 px-3 py-2">
          <div class="font-medium">{faculty.name}</div>
        </div>
        <button class="rounded-md bg-emerald-600/90 hover:bg-emerald-600 active:bg-emerald-700 px-3 py-2 text-xs font-medium transition text-white" on:click={() => onRestore(faculty.id)}>
          {$t('restore')}
        </button>
      </div>
    {/each}
    {#if archived.length === 0}
      <div class="text-sm text-slate-600 dark:text-slate-300">{$t('empty')}</div>
    {/if}
  </div>
</div>


