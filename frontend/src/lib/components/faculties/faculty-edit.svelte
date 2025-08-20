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

  $: if ($faculty && nameDraft === '') {
    nameDraft = $faculty.name
  }

  const save = () => {
    const trimmed = nameDraft.trim()
    if (trimmed.length === 0) return
    if ($faculty) {
      updateFacultyName($faculty.id, trimmed)
    } else {
      addFaculty(trimmed)
    }
    back()
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-300 hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  {#if $faculty}
    <h2 class="text-2xl font-semibold mb-4">{$t('edit_faculty')}</h2>
    <label class="block text-sm mb-2 text-slate-300" for="faculty-name">{$t('faculty_name')}</label>
    <input id="faculty-name" class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={nameDraft} placeholder={$t('faculty_name_placeholder')} />

    <div class="mt-6 flex gap-3">
      <button class="rounded-lg bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 px-4 py-2 text-sm font-medium shadow-sm transition" on:click={save}>
        {$t('save')}
      </button>
      <button class="rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-white/10 transition" on:click={back}>
        {$t('cancel')}
      </button>
    </div>
  {:else}
    <h2 class="text-2xl font-semibold mb-4">{$t('create_faculty')}</h2>
    <label class="block text-sm mb-2 text-slate-300" for="faculty-name">{$t('faculty_name')}</label>
    <input id="faculty-name" class="w-full rounded-lg bg-slate-900/60 focus:bg-slate-900 px-4 py-2 ring-1 ring-white/10 outline-none" bind:value={nameDraft} placeholder={$t('faculty_name_placeholder')} />

    <div class="mt-6 flex gap-3">
      <button class="rounded-lg bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 px-4 py-2 text-sm font-medium shadow-sm transition" on:click={save}>
        {$t('create')}
      </button>
      <button class="rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-white/10 transition" on:click={back}>
        {$t('cancel')}
      </button>
    </div>
  {/if}
</div>


