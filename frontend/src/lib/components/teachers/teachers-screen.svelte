<script lang="ts">
  import { t } from 'svelte-i18n'
  import { teachersStore, type Teacher, deleteTeacher, formatTeacherName } from '../../stores/teachers'
  import { directionsStore } from '../../stores/directions'
  import { navigate } from '../../stores/router'
  import ConfirmModal from '../ui/confirm-modal.svelte'

  const back = () => navigate('home')
  const createNew = () => navigate('teacher_edit', { id: 0 })
  const open = (teacher: Teacher) => navigate('teacher_edit', { id: teacher.id })
  let confirmOpen = false
  let toDeleteId: number | null = null
  const askDelete = (id: number) => { toDeleteId = id; confirmOpen = true }
  const onConfirm = () => { if (toDeleteId != null) deleteTeacher(toDeleteId); confirmOpen = false; toDeleteId = null }
  const onCancel = () => { confirmOpen = false; toDeleteId = null }
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="mb-6 text-slate-300 hover:text-white text-sm" on:click={back}>
    ← {$t('back')}
  </button>

  <h2 class="text-2xl font-semibold">{$t('teachers')}</h2>
  <div class="mt-6 grid grid-cols-1 gap-3">
    {#each $teachersStore as teacher}
      <div class="w-full rounded-lg bg-slate-800/60 px-2 py-2 ring-1 ring-white/10 shadow-sm flex items-center gap-2">
        <button class="flex-1 text-left rounded-md hover:bg-slate-800 active:bg-slate-700 px-3 py-2 transition" on:click={() => open(teacher)}>
          <div class="font-medium">{formatTeacherName(teacher)}</div>
          {#if $directionsStore.find(d => d.id === teacher.directionId)}
            <div class="text-xs text-slate-300">{$directionsStore.find(d => d.id === teacher.directionId).name}</div>
          {/if}
        </button>
        <button class="rounded-md bg-rose-600/90 hover:bg-rose-600 active:bg-rose-700 px-3 py-2 text-xs font-medium transition" on:click={() => askDelete(teacher.id)}>
          {$t('delete')}
        </button>
      </div>
    {/each}
  </div>
  <button class="mt-8 w-full rounded-xl bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 px-4 py-4 text-base md:text-lg font-semibold shadow-md transition" on:click={createNew}>
    {$t('create')}
  </button>
  <ConfirmModal bind:open={confirmOpen} title={$t('confirm_title')} message={$t('confirm_message_generic')} confirmText={$t('confirm')} cancelText={$t('dismiss')} on:confirm={onConfirm} on:cancel={onCancel} />
</div>


