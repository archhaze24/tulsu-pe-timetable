<script lang="ts">
  import { t } from 'svelte-i18n'
  import { directionsStore, updateDirection, createDirection } from '../../stores/directions'
  import { teachersStore, type Teacher, formatTeacherName, updateTeacher } from '../../stores/teachers'
  import { navigate, route } from '../../stores/router'
  import { derived } from 'svelte/store'

  const back = () => navigate('directions')
  const dirId = derived(route, ($route) => Number($route.params?.id ?? 0))
  const direction = derived([directionsStore, dirId], ([$dirs, $id]) => $dirs.find(d => d.id === $id))

  let nameDraft = ''
  let lastInitId = 0
  $: canSave = Boolean(nameDraft.trim())
  let search = ''

  $: {
    const currentId = $direction ? $direction.id : 0
    if ($direction && lastInitId !== currentId) {
      nameDraft = $direction.name
      lastInitId = currentId
    }
    if (!$direction && lastInitId !== 0) {
      lastInitId = 0
    }
  }

  const save = () => {
    const n = nameDraft.trim()
    if (n.length === 0) return
    if ($direction) {
      updateDirection($direction.id, { name: n })
    } else {
      createDirection(n)
    }
    back()
  }

  let filteredTeachers: Teacher[] = []
  $: {
    const q = search.trim().toLowerCase()
    const all = $teachersStore
    filteredTeachers = q
      ? all.filter(t => formatTeacherName(t).toLowerCase().includes(q))
      : all
  }

  const toggle = (teacher: Teacher) => {
    if (!$direction) return
    if (teacher.directionId === $direction.id) {
      updateTeacher(teacher.id, { directionId: 0 })
    } else {
      updateTeacher(teacher.id, { directionId: $direction.id })
    }
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-500 hover:text-slate-700 dark:text-slate-300 dark:hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  {#if $direction}
    <h2 class="text-2xl font-semibold mb-4">{$t('edit_direction')}</h2>
  {:else}
    <h2 class="text-2xl font-semibold mb-4">{$t('create_direction')}</h2>
  {/if}

  <label class="block text-sm mb-2 text-slate-600 dark:text-slate-300" for="dir-name">{$t('direction_name')}</label>
  <input id="dir-name" class="w-full rounded-lg bg-white dark:bg-slate-900/60 focus:bg-white dark:focus:bg-slate-900 px-4 py-2 ring-1 ring-slate-200 dark:ring-white/10 outline-none" bind:value={nameDraft} placeholder={$t('direction_name_placeholder')} />

  

  <div class="mt-6">
    <div class="mb-2 text-sm text-slate-600 dark:text-slate-300">{$t('assign_teachers')}</div>
    <input class="w-full rounded-lg bg-white dark:bg-slate-900/60 focus:bg-white dark:focus:bg-slate-900 px-4 py-2 ring-1 ring-slate-200 dark:ring-white/10 outline-none" bind:value={search} placeholder={$t('search_placeholder')} />
    <div class="mt-3 max-h-60 overflow-auto rounded-md ring-1 ring-slate-200 dark:ring-white/10 divide-y divide-slate-100 dark:divide-white/5">
      {#each filteredTeachers as teacher}
        <button class="w-full flex items-center justify-between px-3 py-2 hover:bg-slate-100 dark:hover:bg-slate-800/60 text-left" on:click={() => toggle(teacher)}>
          <span>{formatTeacherName(teacher)}</span>
          {#if $direction && teacher.directionId === $direction.id}
            <span class="text-emerald-600 dark:text-emerald-400 text-xs">{$t('assigned')}</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <div class="mt-6 flex gap-3">
    <button class="rounded-lg bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed text-white px-4 py-2 text-sm font-medium shadow-sm transition" on:click={save} disabled={!canSave}>
      {$t($direction ? 'save' : 'create')}
    </button>
    <button class="rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-black/5 transition dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 dark:ring-white/10" on:click={back}>
      {$t('cancel')}
    </button>
  </div>
</div>


