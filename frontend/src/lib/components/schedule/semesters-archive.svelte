<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { navigate } from '../../stores/router'
  import { fetchArchivedSemesters, restoreSemester, type Semester } from '../../stores/semesters'
  
  let archived = $state<Semester[]>([])

  const back = () => { navigate('semesters') }

  async function onRestore(id: number) {
    const ok = await restoreSemester(id)
    if (ok) archived = archived.filter(s => s.id !== id)
  }

  onMount(async () => { archived = await fetchArchivedSemesters() })
</script>

<div class="max-w-2xl w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t('semesters')} — {$t('archive')}</h2>

  <div class="grid gap-3">
    {#each archived as s (s.id)}
      <div class="card flex items-center gap-3 px-3 py-2">
        <div class="flex-1">
          <div class="font-medium">{s.name}</div>
          <div class="muted text-xs">{s.startDate} — {s.endDate}</div>
        </div>
        <button class="btn-restore px-3 py-2 text-xs font-medium" onclick={() => onRestore(s.id)}>{$t('restore')}</button>
      </div>
    {/each}
    {#if archived.length === 0}
      <div class="muted text-sm">{$t('empty')}</div>
    {/if}
  </div>
</div>
