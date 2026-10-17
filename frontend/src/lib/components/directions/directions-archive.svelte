<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { navigate } from '../../stores/router'
  import { fetchArchivedDirections, restoreDirection } from '../../stores/directions'
  import type { Direction } from '../../api'
  
  let archived = $state<Direction[]>([])

  const back = () => { navigate('directions') }

  async function onRestore(id: number) {
    if (await restoreDirection(id)) archived = archived.filter(d => d.id !== id)
  }

  onMount(async () => {
    const list = await fetchArchivedDirections()
    if (list) archived = list
  })
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t('directions')} — {$t('archive')}</h2>

  <div class="mt-4 grid grid-cols-1 gap-3">
    {#each archived as dir (dir.id)}
      <div class="card flex w-full items-center gap-2 px-2 py-2 shadow-sm">
        <div class="flex-1 px-3 py-2">
          <div class="font-medium">{dir.name}</div>
        </div>
        <button class="btn-restore px-3 py-2 text-xs font-medium" onclick={() => onRestore(dir.id)}>{$t('restore')}</button>
      </div>
    {/each}
    {#if archived.length === 0}
      <div class="muted text-sm">{$t('empty')}</div>
    {/if}
  </div>
</div>
