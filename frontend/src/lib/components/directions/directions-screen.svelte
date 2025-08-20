<script lang="ts">
  import { t } from 'svelte-i18n'
  import { directionsStore, deleteDirection, type Direction } from '../../stores/directions'
  import { navigate } from '../../stores/router'

  const back = () => navigate('home')
  const createNew = () => navigate('direction_edit', { id: 0 })
  const open = (dir: Direction) => navigate('direction_edit', { id: dir.id })
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-300 hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  <h2 class="text-2xl font-semibold">{$t('directions')}</h2>
  <div class="mt-6 grid grid-cols-1 gap-3">
    {#each $directionsStore as dir}
      <div class="w-full rounded-lg bg-slate-800/60 px-2 py-2 ring-1 ring-white/10 shadow-sm flex items-center gap-2">
        <button class="flex-1 text-left rounded-md hover:bg-slate-800 active:bg-slate-700 px-3 py-2 transition" on:click={() => open(dir)}>
          <div class="font-medium">{dir.name}</div>
          {#if dir.address}
            <div class="text-xs text-slate-300">{dir.address}</div>
          {/if}
        </button>
        <button class="rounded-md bg-rose-600/90 hover:bg-rose-600 active:bg-rose-700 px-3 py-2 text-xs font-medium transition" on:click={() => deleteDirection(dir.id)}>
          {$t('delete')}
        </button>
      </div>
    {/each}
  </div>
  <button class="mt-8 w-full rounded-xl bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 px-4 py-4 text-base md:text-lg font-semibold shadow-md transition" on:click={createNew}>
    {$t('create')}
  </button>
</div>


