<script lang="ts">
  import { t } from 'svelte-i18n'
  import { directionsStore, updateDirection, addDirection, toggleTeacher } from '../../stores/directions'
  import { teachersStore, type Teacher } from '../../stores/teachers'
  import { navigate, route } from '../../stores/router'
  import { derived } from 'svelte/store'

  const back = () => navigate('directions')
  const dirId = derived(route, ($route) => Number($route.params?.id ?? 0))
  const direction = derived([directionsStore, dirId], ([$dirs, $id]) => $dirs.find(d => d.id === $id))

  let nameDraft = ''
  let addressDraft = ''
  let search = ''

  $: if ($direction && nameDraft === '' && addressDraft === '') {
    nameDraft = $direction.name
    addressDraft = $direction.address ?? ''
  }

  const save = () => {
    const n = nameDraft.trim()
    const a = addressDraft.trim() || undefined
    if (n.length === 0) return
    if ($direction) {
      updateDirection($direction.id, { name: n, address: a })
    } else {
      addDirection(n, a)
    }
    back()
  }

  const filteredTeachers = derived(teachersStore, ($t) => {
    const q = search.trim().toLowerCase()
    if (!q) return $t
    return $t.filter(t => t.name.toLowerCase().includes(q))
  })

  const toggle = (teacher: Teacher) => {
    if ($direction) toggleTeacher($direction.id, teacher.id)
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-300 hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  {#if $direction}
    <h2 class="text-2xl font-semibold mb-4">{$t('edit_direction')}</h2>
  {:else}
    <h2 class="text-2xl font-semibold mb-4">{$t('create_direction')}</h2>
  {/if}

  <label class="block text-sm mb-2 text-slate-300" for="dir-name">{$t('direction_name')}</label>
  <input id="dir-name" class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={nameDraft} placeholder={$t('direction_name_placeholder')} />

  <label class="block text-sm mb-2 mt-4 text-slate-300" for="dir-address">{$t('direction_address')}</label>
  <input id="dir-address" class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={addressDraft} placeholder={$t('direction_address_placeholder')} />

  <div class="mt-6">
    <div class="mb-2 text-sm text-slate-300">{$t('assign_teachers')}</div>
    <input class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={search} placeholder={$t('search_placeholder')} />
    <div class="mt-3 max-h-60 overflow-auto rounded-md ring-1 ring-white/10 divide-y divide-white/5">
      {#each $filteredTeachers as teacher}
        <button class="w-full flex items-center justify-between px-3 py-2 hover:bg-slate-800/60 text-left" on:click={() => toggle(teacher)}>
          <span>{teacher.name}</span>
          {#if $direction && $direction.teacherIds.includes(teacher.id)}
            <span class="text-emerald-400 text-xs">{$t('assigned')}</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <div class="mt-6 flex gap-3">
    <button class="rounded-lg bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 px-4 py-2 text-sm font-medium shadow-sm transition" on:click={save}>
      {$t($direction ? 'save' : 'create')}
    </button>
    <button class="rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-white/10 transition" on:click={back}>
      {$t('cancel')}
    </button>
  </div>
</div>


