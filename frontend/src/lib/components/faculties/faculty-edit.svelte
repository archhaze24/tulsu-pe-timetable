<script lang="ts">
  import { t } from 'svelte-i18n'
  import { facultiesStore, updateFacultyName, addFaculty, } from '../../stores/faculties'
  import { navigate, route } from '../../stores/router'
  import { derived } from 'svelte/store'

  const back = () => navigate('faculties')

  const facultyId = derived(route, ($route) => Number($route.params?.id ?? 0))
  const faculty = derived([facultiesStore, facultyId], ([$faculties, $id]) =>
    $faculties.find(f => f.id === $id)
  )

  let nameDraft = ''
  let lastInitId = 0
  $: canSave = Boolean(nameDraft.trim())

  $: {
    const currentId = $faculty ? $faculty.id : 0
    if ($faculty && lastInitId !== currentId) {
      nameDraft = $faculty.name
      lastInitId = currentId
    }
    if (!$faculty && lastInitId !== 0) {
      lastInitId = 0
    }
  }

  const save = async () => {
    const trimmed = nameDraft.trim()
    if (trimmed.length === 0) return
    if ($faculty) {
      await updateFacultyName($faculty.id, trimmed)
    } else {
      await addFaculty(trimmed)
    }
    back()
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-500 hover:text-slate-700 dark:text-slate-300 dark:hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  {#if $faculty}
    <h2 class="text-2xl font-semibold mb-4">{$t('edit_faculty')}</h2>
    <label class="block text-sm mb-2 text-slate-600 dark:text-slate-300" for="faculty-name">{$t('faculty_name')}</label>
    <input id="faculty-name" class="w-full rounded-lg bg-white dark:bg-slate-900/60 focus:bg-white dark:focus:bg-slate-900 px-4 py-2 ring-1 ring-slate-200 dark:ring-white/10 outline-none" bind:value={nameDraft} placeholder={$t('faculty_name_placeholder')} />

    <div class="mt-6 flex gap-3">
      <button class="rounded-lg bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed text-white px-4 py-2 text-sm font-medium shadow-sm transition" on:click={save} disabled={!canSave}>
        {$t('save')}
      </button>
      <button class="rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-black/5 transition dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 dark:ring-white/10" on:click={back}>
        {$t('cancel')}
      </button>
    </div>
  {:else}
    <h2 class="text-2xl font-semibold mb-4">{$t('create_faculty')}</h2>
    <label class="block text-sm mb-2 text-slate-600 dark:text-slate-300" for="faculty-name">{$t('faculty_name')}</label>
    <input id="faculty-name" class="w-full rounded-lg bg-white dark:bg-slate-900/60 focus:bg-white dark:focus:bg-slate-900 px-4 py-2 ring-1 ring-slate-200 dark:ring-white/10 outline-none" bind:value={nameDraft} placeholder={$t('faculty_name_placeholder')} />

    <div class="mt-6 flex gap-3">
      <button class="rounded-lg bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed text-white px-4 py-2 text-sm font-medium shadow-sm transition" on:click={save} disabled={!canSave}>
        {$t('create')}
      </button>
      <button class="rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-black/5 transition dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 dark:ring-white/10" on:click={back}>
        {$t('cancel')}
      </button>
    </div>
  {/if}
</div>


