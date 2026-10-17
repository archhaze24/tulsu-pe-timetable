<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { navigate } from '../../stores/router'
  import { fetchArchivedFaculties, restoreFaculty } from '../../stores/faculties'
  import type { Faculty } from '../../api'
  
  let archived = $state<Faculty[]>([])

  const back = () => { navigate('faculties') }

  async function onRestore(id: number) {
    if (await restoreFaculty(id)) archived = archived.filter(f => f.id !== id)
  }

  onMount(async () => {
    const list = await fetchArchivedFaculties()
    if (list) archived = list
  })
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t('faculties')} — {$t('archive')}</h2>

  <div class="mt-4 grid grid-cols-1 gap-3">
    {#each archived as faculty (faculty.id)}
      <div class="card flex w-full items-center gap-2 px-2 py-2 shadow-sm">
        <div class="flex-1 px-3 py-2">
          <div class="font-medium">{faculty.name}</div>
        </div>
        <button class="btn-restore px-3 py-2 text-xs font-medium" onclick={() => onRestore(faculty.id)}>{$t('restore')}</button>
      </div>
    {/each}
    {#if archived.length === 0}
      <div class="muted text-sm">{$t('empty')}</div>
    {/if}
  </div>
</div>
