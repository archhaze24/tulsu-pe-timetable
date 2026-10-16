<script lang="ts">
  import { t } from 'svelte-i18n'
  import { derived } from 'svelte/store'
  import { facultiesStore, updateFacultyName, addFaculty } from '../../stores/faculties'
  import { navigate, route } from '../../stores/router'
  
  const facultyId = derived(route, ($route) => Number($route.params?.id ?? 0))
  const faculty = derived([facultiesStore, facultyId], ([$faculties, $id]) => $faculties.find(f => f.id === $id))

  let nameDraft = $state('')
  let lastInitId = 0
  const canSave = $derived(Boolean(nameDraft.trim()))

  $effect(() => {
    const fac = $faculty
    if (fac && lastInitId !== fac.id) {
      nameDraft = fac.name
      lastInitId = fac.id
    }
    if (!fac) lastInitId = 0
  })

  const back = () => { navigate('faculties') }

  const save = async () => {
    const name = nameDraft.trim()
    if (!name) return
    if ($faculty) {
      await updateFacultyName($faculty.id, name)
    } else {
      await addFaculty(name)
    }
    back()
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t($faculty ? 'edit_faculty' : 'create_faculty')}</h2>
  <label class="field-label mb-2 block" for="faculty-name">{$t('faculty_name')}</label>
  <input id="faculty-name" class="input" bind:value={nameDraft} placeholder={$t('faculty_name_placeholder')} />

  <div class="mt-6 flex gap-3">
    <button class="btn-success px-4 py-2 text-sm font-medium shadow-sm" onclick={save} disabled={!canSave}>
      {$t($faculty ? 'save' : 'create')}
    </button>
    <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={back}>{$t('cancel')}</button>
  </div>
</div>
