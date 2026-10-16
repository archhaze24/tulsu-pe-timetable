<script lang="ts">
  import { t } from 'svelte-i18n'
  import { derived } from 'svelte/store'
  import { directionsStore, updateDirection, createDirection } from '../../stores/directions'
  import { teachersStore, type Teacher, formatTeacherName, updateTeacher } from '../../stores/teachers'
  import { navigate, route } from '../../stores/router'
  
  const dirId = derived(route, ($route) => Number($route.params?.id ?? 0))
  const direction = derived([directionsStore, dirId], ([$dirs, $id]) => $dirs.find(d => d.id === $id))

  let nameDraft = $state('')
  let search = $state('')
  // Режим создания: выбранные преподаватели назначатся новому направлению после его создания.
  let selectedCreateIds = $state(new Set<number>())
  let lastInitId = 0

  const canSave = $derived(Boolean(nameDraft.trim()))

  $effect(() => {
    const dir = $direction
    if (dir && lastInitId !== dir.id) {
      nameDraft = dir.name
      lastInitId = dir.id
    }
    if (!dir) lastInitId = 0
  })

  const filteredTeachers = $derived.by(() => {
    const q = search.trim().toLowerCase()
    return q ? $teachersStore.filter(t => formatTeacherName(t).toLowerCase().includes(q)) : $teachersStore
  })

  const back = () => { navigate('directions') }

  const save = async () => {
    const name = nameDraft.trim()
    if (!name) return
    if ($direction) {
      await updateDirection($direction.id, name)
    } else {
      const created = await createDirection(name)
      if (created) {
        for (const teacherId of selectedCreateIds) {
          await updateTeacher(teacherId, { directionId: created.id })
        }
      }
    }
    back()
  }

  // Преподаватель из чужого направления здесь недоступен: сначала его нужно снять в том направлении.
  const canToggle = (teacher: Teacher) => {
    if (!$direction) return false
    return teacher.directionId === 0 || teacher.directionId === $direction.id
  }

  const toggle = (teacher: Teacher) => {
    if (!$direction || !canToggle(teacher)) return
    const assigned = teacher.directionId === $direction.id
    updateTeacher(teacher.id, { directionId: assigned ? 0 : $direction.id })
  }

  const toggleSelectCreate = (teacher: Teacher) => {
    if (teacher.directionId !== 0) return
    const next = new Set(selectedCreateIds)
    if (next.has(teacher.id)) next.delete(teacher.id)
    else next.add(teacher.id)
    selectedCreateIds = next
  }

  const rowClass = 'flex w-full items-center justify-between px-3 py-2 text-left hover:bg-slate-100 disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-transparent dark:hover:bg-slate-800/60'
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t($direction ? 'edit_direction' : 'create_direction')}</h2>

  <label class="field-label mb-2 block" for="dir-name">{$t('direction_name')}</label>
  <input id="dir-name" class="input" bind:value={nameDraft} placeholder={$t('direction_name_placeholder')} />

  <div class="mt-6">
    <div class="muted mb-2 text-sm">{$t('assign_teachers')}</div>
    <input class="input" bind:value={search} placeholder={$t('search_placeholder')} />
    <div class="mt-3 max-h-60 divide-y divide-slate-100 overflow-auto rounded-md ring-1 ring-slate-200 dark:divide-white/5 dark:ring-white/10">
      {#each filteredTeachers as teacher (teacher.id)}
        {#if $direction}
          <button class={rowClass} onclick={() => toggle(teacher)} disabled={!canToggle(teacher)}>
            <span>{formatTeacherName(teacher)}</span>
            {#if teacher.directionId === $direction.id}
              <span class="text-xs text-emerald-600 dark:text-emerald-400">{$t('assigned')}</span>
            {:else if teacher.directionId !== 0}
              <span class="text-xs text-amber-600 dark:text-amber-400">{$t('assigned_to_direction')}: {teacher.directionName}</span>
            {/if}
          </button>
        {:else}
          <button class={rowClass} onclick={() => toggleSelectCreate(teacher)} disabled={teacher.directionId !== 0}>
            <span>{formatTeacherName(teacher)}</span>
            {#if teacher.directionId !== 0}
              <span class="text-xs text-amber-600 dark:text-amber-400">{$t('assigned_to_direction')}: {teacher.directionName}</span>
            {:else if selectedCreateIds.has(teacher.id)}
              <span class="text-xs text-emerald-600 dark:text-emerald-400">{$t('will_assign')}</span>
            {/if}
          </button>
        {/if}
      {/each}
    </div>
  </div>

  <div class="mt-6 flex gap-3">
    <button class="btn-success px-4 py-2 text-sm font-medium shadow-sm" onclick={save} disabled={!canSave}>
      {$t($direction ? 'save' : 'create')}
    </button>
    <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={back}>{$t('cancel')}</button>
  </div>
</div>
