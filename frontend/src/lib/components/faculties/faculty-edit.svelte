<script lang="ts">
  import { t } from 'svelte-i18n'
  import { facultiesStore, updateFacultyName, addFaculty } from '../../stores/faculties'
  import { navigate, route } from '../../stores/router'

  const facultyId = $derived(Number($route.params?.id ?? 0))
  const isEdit = $derived(facultyId > 0)
  const faculty = $derived($facultiesStore.find(f => f.id === facultyId))
  const notFound = $derived(isEdit && !faculty)

  let nameDraft = $state('')
  let lastInitId = 0
  const nameEmpty = $derived(!nameDraft.trim())
  const canSave = $derived(!notFound && !nameEmpty)

  $effect(() => {
    const fac = faculty
    if (fac && lastInitId !== fac.id) {
      nameDraft = fac.name
      lastInitId = fac.id
    }
    if (!fac) lastInitId = 0
  })

  const back = () => { navigate('faculties') }

  const save = async () => {
    if (!canSave) return
    const name = nameDraft.trim()
    const done = isEdit ? await updateFacultyName(facultyId, name) : (await addFaculty(name)) !== null
    if (done) back()
  }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t(isEdit ? 'edit_faculty' : 'create_faculty')}</h2>

  {#if notFound}
    <div class="mb-4 text-sm text-rose-600">{$t('errors.faculties.not_found')}</div>
  {/if}

  <label class="field-label mb-2 block" for="faculty-name">{$t('faculty_name')}</label>
  <input id="faculty-name" class="input" bind:value={nameDraft} placeholder={$t('faculty_name_placeholder')} disabled={notFound} />
  {#if nameEmpty && !notFound}
    <div class="mt-1 text-xs text-rose-600">{$t('validation.name_required')}</div>
  {/if}

  <div class="mt-6 flex gap-3">
    <button class="btn-success px-4 py-2 text-sm font-medium shadow-sm" onclick={save} disabled={!canSave}>
      {$t(isEdit ? 'save' : 'create')}
    </button>
    <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={back}>{$t('cancel')}</button>
  </div>
</div>
