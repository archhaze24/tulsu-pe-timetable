<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { directionsStore, saveDirection } from '../../stores/directions'
  import { teachersStore, formatTeacherName, refreshTeachers } from '../../stores/teachers'
  import { navigate, route } from '../../stores/router'
  import type { Teacher } from '../../api'

  const dirId = $derived(Number($route.params?.id ?? 0))
  const isEdit = $derived(dirId > 0)
  const direction = $derived($directionsStore.find(d => d.id === dirId))
  const notFound = $derived(isEdit && !direction)

  let nameDraft = $state('')
  let search = $state('')
  // Набор преподавателей меняется только локально и уходит на бэкенд одним запросом при сохранении.
  let selectedIds = $state(new Set<number>())
  // Без загруженного списка преподавателей сохранять нельзя: пустой набор снял бы всех.
  let ready = $state(false)
  let lastInitId = 0

  const nameEmpty = $derived(!nameDraft.trim())
  const canSave = $derived(ready && !notFound && !nameEmpty)

  $effect(() => {
    const dir = direction
    if (dir && lastInitId !== dir.id) {
      nameDraft = dir.name
      lastInitId = dir.id
    }
    if (!dir) lastInitId = 0
  })

  onMount(async () => {
    if (!(await refreshTeachers())) return
    selectedIds = new Set(isEdit ? $teachersStore.filter(t => t.directionId === dirId).map(t => t.id) : [])
    ready = true
  })

  const filteredTeachers = $derived.by(() => {
    const q = search.trim().toLowerCase()
    return q ? $teachersStore.filter(t => formatTeacherName(t).toLowerCase().includes(q)) : $teachersStore
  })

  const back = () => { navigate('directions') }

  // Преподаватель из чужого направления здесь недоступен: сначала его нужно перевести или открепить там.
  const canSelect = (teacher: Teacher) => teacher.directionId === 0 || teacher.directionId === dirId

  const toggle = (teacher: Teacher) => {
    if (!canSelect(teacher)) return
    const next = new Set(selectedIds)
    if (next.has(teacher.id)) next.delete(teacher.id)
    else next.add(teacher.id)
    selectedIds = next
  }

  const save = async () => {
    if (!canSave) return
    const saved = await saveDirection({
      id: isEdit ? dirId : undefined,
      name: nameDraft.trim(),
      teacherIds: [...selectedIds]
    })
    if (saved) back()
  }

  const rowClass = 'flex w-full flex-col items-start gap-0.5 px-3 py-2 text-left hover:bg-slate-100 disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-transparent'
</script>

<div class="max-w-md w-full px-6 py-10">
  <button class="back-link mb-6" onclick={back}>← {$t('back')}</button>

  <h2 class="mb-4 text-2xl font-semibold">{$t(isEdit ? 'edit_direction' : 'create_direction')}</h2>

  {#if notFound}
    <div class="mb-4 text-sm text-rose-600">{$t('errors.directions.not_found')}</div>
  {/if}

  <label class="field-label mb-2 block" for="dir-name">{$t('direction_name')}</label>
  <input id="dir-name" class="input" bind:value={nameDraft} placeholder={$t('direction_name_placeholder')} disabled={notFound} />
  {#if nameEmpty && !notFound}
    <div class="mt-1 text-xs text-rose-600">{$t('validation.name_required')}</div>
  {/if}

  <div class="mt-6">
    <div class="muted mb-2 text-sm">{$t('assign_teachers')}</div>
    <input class="input" bind:value={search} placeholder={$t('search_placeholder')} />
    <div class="mt-3 grid max-h-60 gap-1 overflow-auto rounded-md p-1 ring-1 ring-slate-200">
      {#each filteredTeachers as teacher (teacher.id)}
        {@const selectable = canSelect(teacher)}
        <button class={rowClass} onclick={() => toggle(teacher)} disabled={!selectable}>
          <span>{formatTeacherName(teacher)}</span>
          {#if selectedIds.has(teacher.id)}
            <span class="text-xs text-emerald-600">{$t('assigned')}</span>
          {:else if !selectable}
            <span class="text-xs text-amber-600">{$t('assigned_to_direction')}: {teacher.directionName}{#if teacher.directionArchived} {$t('archived_suffix')}{/if}</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <div class="mt-6 flex gap-3">
    <button class="btn-success px-4 py-2 text-sm font-medium shadow-sm" onclick={save} disabled={!canSave}>
      {$t(isEdit ? 'save' : 'create')}
    </button>
    <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={back}>{$t('cancel')}</button>
  </div>
</div>
