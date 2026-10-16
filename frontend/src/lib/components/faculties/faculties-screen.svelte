<script lang="ts">
  import { t } from 'svelte-i18n'

  import { facultiesStore, type Faculty, deleteFaculty } from '../../stores/faculties'
  import { navigate } from '../../stores/router'
  import ConfirmModal from '../ui/confirm-modal.svelte'
  
  let confirmOpen = $state(false)
  let toDeleteId = $state<number | null>(null)

  const openFaculty = (faculty: Faculty) => {
    navigate('faculty_edit', { id: faculty.id })
  }
  const back = () => { navigate('home') }
  const createNew = () => { navigate('faculty_edit', { id: 0 }) }
  const openArchive = () => { navigate('faculties_archive') }
  const askDelete = (id: number) => { toDeleteId = id; confirmOpen = true }
  const onConfirm = () => {
    if (toDeleteId != null) deleteFaculty(toDeleteId)
    confirmOpen = false
    toDeleteId = null
  }
  const onCancel = () => { confirmOpen = false; toDeleteId = null }

</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <div class="flex items-center justify-between gap-3">
    <h2 class="text-2xl font-semibold">{$t('faculties')}</h2>
    <button class="btn-neutral px-3 py-2 text-xs font-medium shadow-sm" onclick={openArchive}>{$t('archive')}</button>
  </div>

  <div class="mt-6 grid grid-cols-1 gap-3">
    {#each $facultiesStore as faculty (faculty.id)}
      <div class="card flex w-full items-center gap-2 px-2 py-2 shadow-sm">
        <button class="row-item" onclick={() => openFaculty(faculty)}>
          {faculty.name}
        </button>
        <button class="btn-danger px-3 py-2 text-xs font-medium" onclick={() => askDelete(faculty.id)}>{$t('delete')}</button>
      </div>
    {/each}
  </div>

  <button class="btn-success mt-8 w-full rounded-xl px-4 py-4 text-base font-semibold shadow-md md:text-lg" onclick={createNew}>
    {$t('create')}
  </button>
  <ConfirmModal open={confirmOpen} title={$t('confirm_title')} message={$t('confirm_message_generic')} confirmText={$t('confirm')} cancelText={$t('dismiss')} onconfirm={onConfirm} oncancel={onCancel} />
</div>
