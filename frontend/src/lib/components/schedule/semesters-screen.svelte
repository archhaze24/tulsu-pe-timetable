<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { navigate } from '../../stores/router'
  import { semestersStore, loadSemesters, deleteSemester } from '../../stores/semesters'
  import type { Semester } from '../../api'
  import ConfirmModal from '../ui/confirm-modal.svelte'

  let confirmOpen = $state(false)
  let toDeleteId = $state<number | null>(null)

  const back = () => { navigate('home') }
  const createNew = () => { navigate('semester_edit', { id: 0 }) }
  const openArchive = () => { navigate('semesters_archive') }
  const edit = (s: Semester) => { navigate('semester_edit', { id: s.id }) }
  const open = (s: Semester) => { navigate('schedule', { id: s.id }) }

  const askDelete = (id: number) => { toDeleteId = id; confirmOpen = true }
  const onConfirm = async () => {
    const id = toDeleteId
    if (id == null) return
    toDeleteId = null
    await deleteSemester(id)
    confirmOpen = false
  }
  const onCancel = () => { confirmOpen = false; toDeleteId = null }

  onMount(() => { loadSemesters() })
</script>

<div class="max-w-2xl w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <div class="mb-4 flex items-center justify-between gap-3">
    <h2 class="text-2xl font-semibold">{$t('semesters')}</h2>
    <div class="flex items-center gap-2">
      <button class="btn-neutral inline-flex h-10 items-center px-4 text-sm font-medium shadow-sm" onclick={openArchive}>{$t('archive')}</button>
      <button class="btn-success inline-flex h-10 items-center px-4 text-sm font-medium shadow-sm" onclick={createNew}>{$t('new_semester')}</button>
    </div>
  </div>

  <div class="grid gap-3">
    {#each $semestersStore as s (s.id)}
      <div class="card flex items-center gap-3 px-3 py-2">
        <div class="flex-1">
          <div class="font-medium">{s.name}</div>
          <div class="muted text-xs">{s.startDate} — {s.endDate}</div>
        </div>
        <div class="flex items-center gap-2">
          <button class="btn-neutral px-3 py-2 text-xs font-medium" onclick={() => edit(s)}>{$t('edit')}</button>
          <button class="rounded-md bg-indigo-600/90 px-3 py-2 text-xs font-medium text-white transition hover:bg-indigo-600 active:bg-indigo-700" onclick={() => open(s)}>
            {$t('open_schedule')}
          </button>
          <button class="btn-danger px-3 py-2 text-xs font-medium" onclick={() => askDelete(s.id)}>{$t('confirm_archive')}</button>
        </div>
      </div>
    {/each}
  </div>
  <ConfirmModal open={confirmOpen} title={$t('confirm_title')} message={$t('confirm_archive_message')} confirmText={$t('confirm_archive')} cancelText={$t('cancel')} onconfirm={onConfirm} oncancel={onCancel} />
</div>
