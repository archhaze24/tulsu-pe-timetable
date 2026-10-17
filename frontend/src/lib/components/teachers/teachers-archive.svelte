<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { navigate } from '../../stores/router'
  import { fetchArchivedTeachers, restoreTeacher, formatTeacherName } from '../../stores/teachers'
  import type { Teacher } from '../../api'
  
  let archived = $state<Teacher[]>([])

  const back = () => { navigate('teachers') }

  async function onRestore(id: number) {
    if (await restoreTeacher(id)) archived = archived.filter(t => t.id !== id)
  }

  onMount(async () => {
    const list = await fetchArchivedTeachers()
    if (list) archived = list
  })
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t('teachers')} — {$t('archive')}</h2>

  <div class="mt-4 grid grid-cols-1 gap-3">
    {#each archived as teacher (teacher.id)}
      <div class="card flex w-full items-center gap-2 px-2 py-2 shadow-sm">
        <div class="flex-1 px-3 py-2">
          <div class="font-medium">{formatTeacherName(teacher)}</div>
        </div>
        <button class="btn-restore px-3 py-2 text-xs font-medium" onclick={() => onRestore(teacher.id)}>{$t('restore')}</button>
      </div>
    {/each}
    {#if archived.length === 0}
      <div class="muted text-sm">{$t('empty')}</div>
    {/if}
  </div>
</div>
