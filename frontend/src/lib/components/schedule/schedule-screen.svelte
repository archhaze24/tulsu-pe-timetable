<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { save } from '@tauri-apps/plugin-dialog'
  import * as App from '../../api'
  import type { ScheduleView, SlotView, ScheduleOutcome, TeacherView, TimePair, Teacher, UnbindResult } from '../../api'
  import { navigate, route } from '../../stores/router'
  import { semestersStore, loadSemesters } from '../../stores/semesters'
  import { formatTeacherName } from '../../stores/teachers'
  import { request, requestFlag, translateError } from '../../stores/request'
  import { notify } from '../../stores/notify'
  import { loadSemesterTeachers, bindTeacherToSemester, previewUnbindTeacher, unbindTeacherFromSemester, semesterTeachersStore } from '../../stores/semester-teachers'
  import ConfirmModal from '../ui/confirm-modal.svelte'
  import { overlay } from '../ui/overlay'

  const semesterId = $derived(Number($route.params?.id ?? 0) || $semestersStore[0]?.id || 0)
  const semester = $derived($semestersStore.find(s => s.id === semesterId))

  let view = $state<ScheduleView | null>(null)
  let loadError = $state('')

  const back = () => { navigate('semesters') }

  // При ошибке уже показанное расписание остаётся, рядом выводится сообщение.
  async function reload(id: number): Promise<void> {
    try {
      const resp = await App.GetSchedule(id)
      if (id !== semesterId) return
      if (resp.error) {
        loadError = translateError(resp.error)
        return
      }
      loadError = ''
      view = resp.data
    } catch (e) {
      console.error('GetSchedule failed:', e)
      if (id === semesterId) loadError = $t('errors.unknown')
    }
  }

  const refresh = () => reload(semesterId)

  onMount(() => { loadSemesters() })

  $effect(() => {
    const id = semesterId
    if (!id) return
    view = null
    loadError = ''
    reload(id)
    loadSemesterTeachers(id)
  })

  const days = $derived([
    { key: 1, label: $t('mon') },
    { key: 2, label: $t('tue') },
    { key: 3, label: $t('wed') },
    { key: 4, label: $t('thu') },
    { key: 5, label: $t('fri') },
    { key: 6, label: $t('sat') },
    { key: 7, label: $t('sun') },
  ])

  // ----- Индексы по модели: считаются один раз на обновление, а не в каждой ячейке -----

  const slotKey = (day: number, slot: string) => `${day}|${slot}`

  const slotsOf = (day: number): SlotView[] => view?.days.find(d => d.day === day)?.slots ?? []

  const slotByKey = $derived(new Map<string, SlotView>(
    (view?.days ?? []).flatMap(d => d.slots.map(s => [slotKey(d.day, s.slot), s] as const))
  ))

  const facultyById = $derived(new Map((view?.faculties ?? []).map(f => [f.id, f] as const)))

  const dayRows = $derived(days.map(d => ({
    ...d,
    rows: slotsOf(d.key).map(slot => ({ slot, assigned: new Set(slot.assignedTeacherIds) }))
  })))

  // ----- Ответы бэкенда -----

  // Бэкенд возвращает коды; тексты берём из локалей фронтенда.
  function outcomeMessage(outcome: Exclude<ScheduleOutcome, { code: 'ok' }>): string {
    switch (outcome.code) {
      case 'teacher_count_exceeded': return $t('outcome.teacher_count_exceeded', { values: { max: outcome.max } })
      case 'teacher_count_below_assigned': return $t('outcome.teacher_count_below_assigned', { values: { assigned: outcome.assigned } })
      default: return $t(`outcome.${outcome.code}`)
    }
  }

  // true — операция выполнена; иначе показывает перевод кода.
  function checkOutcome(outcome: ScheduleOutcome): boolean {
    if (outcome.code === 'ok') return true
    notify(outcomeMessage(outcome))
    return false
  }

  // Повторный клик, пока идёт запрос с тем же ключом, не отправляет вторую операцию.
  const pending = new Set<string>()
  async function once(key: string, action: () => Promise<void>): Promise<void> {
    if (pending.has(key)) return
    pending.add(key)
    try {
      await action()
    } finally {
      pending.delete(key)
    }
  }

  async function toggleTeacher(day: number, slot: string, teacherId: number) {
    await once(`cell|${slotKey(day, slot)}|${teacherId}`, async () => {
      const outcome = await request('ToggleTeacherInSlot', () => App.ToggleTeacherInSlot({ semesterId, day, slot, teacherId }))
      if (outcome && checkOutcome(outcome)) await refresh()
    })
  }

  // ----- Позиционирование попапов -----

  type Point = { x: number; y: number }
  const POPOVER_WIDTH = 360
  const EDGE_GAP = 8

  const popoverLeft = (rect: DOMRect): number => {
    const baseLeft = rect.left + rect.width / 2 - POPOVER_WIDTH / 2
    return Math.max(EDGE_GAP, Math.min(window.innerWidth - POPOVER_WIDTH - EDGE_GAP, baseLeft))
  }

  // ----- Добавление пары -----

  let addSlotPopover = $state<Point | null>(null)
  let addSlotDay = $state(1)
  let addSlotPairKey = $state('')
  let addSlotCustom = $state(false)
  let addSlotStart = $state('09:40')
  let addSlotEnd = $state('11:15')
  let addSlotTeacherCount = $state<number | null>(null)
  let addSlotHint = $state('')

  const pairKey = (p: TimePair) => `${p.startTime}-${p.endTime}`
  const hasSlotTime = (day: number, p: TimePair) =>
    slotsOf(day).some(s => s.startTime === p.startTime && s.endTime === p.endTime)

  // Стандартные пары, которых ещё нет в выбранном дне.
  const freePairs = $derived((view?.standardPairs ?? []).filter(p => !hasSlotTime(addSlotDay, p)))

  const pickedTime = (): TimePair | null => {
    if (!addSlotCustom) return freePairs.find(p => pairKey(p) === addSlotPairKey) ?? null
    return addSlotStart && addSlotEnd ? { startTime: addSlotStart, endTime: addSlotEnd } : null
  }

  function openAddSlot(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    addSlotHint = ''
    addSlotPopover = { x: popoverLeft(rect), y: rect.bottom + EDGE_GAP }
  }

  function closeAddSlot() {
    addSlotPopover = null
  }

  function setAddSlotDay(day: number) {
    addSlotDay = day
    const picked = view?.standardPairs.find(p => pairKey(p) === addSlotPairKey)
    if (picked && hasSlotTime(day, picked)) addSlotPairKey = ''
  }

  async function createTimeSlot() {
    const time = pickedTime()
    if (!time) {
      addSlotHint = addSlotCustom ? $t('validation.time_required') : $t('validation.pair_required')
      return
    }
    const teacherCount = addSlotTeacherCount
    if (teacherCount === null || !Number.isInteger(teacherCount) || teacherCount < 1) {
      addSlotHint = $t('outcome.teacher_count_invalid')
      return
    }
    addSlotHint = ''
    const day = addSlotDay
    const customTime = addSlotCustom
    await once('add-slot', async () => {
      const outcome = await request('AddSlot', () => App.AddSlot({
        semesterId,
        day,
        startTime: time.startTime,
        endTime: time.endTime,
        teacherCount,
        customTime
      }))
      if (outcome && checkOutcome(outcome)) {
        addSlotPopover = null
        await refresh()
      }
    })
  }

  // ----- Факультеты строки -----

  let facultyPopover = $state<{ day: number; slot: string; pos: Point } | null>(null)

  // Архивные факультеты скрыты, кроме уже выбранных: их можно снять.
  const popoverFaculties = $derived.by(() => {
    if (!view || !facultyPopover) return []
    const selected = new Set<number>(slotByKey.get(slotKey(facultyPopover.day, facultyPopover.slot))?.facultyIds)
    return view.faculties
      .filter(f => !f.isArchived || selected.has(f.id))
      .map(f => ({ ...f, checked: selected.has(f.id) }))
  })

  function openFacultyPopover(e: MouseEvent, day: number, slot: string) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const estimatedHeight = 320
    const openBelow = window.innerHeight - rect.bottom >= estimatedHeight + EDGE_GAP
    const top = openBelow ? rect.bottom + EDGE_GAP : Math.max(EDGE_GAP, rect.top - estimatedHeight - EDGE_GAP)
    facultyPopover = { day, slot, pos: { x: popoverLeft(rect), y: top } }
  }

  function closeFacultyPopover() {
    facultyPopover = null
  }

  async function toggleRowFaculty(facultyId: number) {
    const popover = facultyPopover
    if (!popover) return
    const { day, slot } = popover
    const current = slotByKey.get(slotKey(day, slot))?.facultyIds ?? []
    const next = current.includes(facultyId) ? current.filter(id => id !== facultyId) : [...current, facultyId]
    await once(`row|${slotKey(day, slot)}`, async () => {
      if (await requestFlag('SetSlotFaculties', () => App.SetSlotFaculties({ semesterId, day, slot, facultyIds: next }))) {
        await refresh()
      }
    })
  }

  // ----- Лимит преподавателей слота -----

  let limitTarget = $state<{ day: number; slot: string; pos: Point } | null>(null)
  let limitValue = $state<number | null>(null)
  let limitHint = $state('')

  function openLimit(e: MouseEvent, day: number, slot: SlotView) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    limitValue = slot.teacherMax
    limitHint = ''
    limitTarget = { day, slot: slot.slot, pos: { x: popoverLeft(rect), y: rect.bottom + EDGE_GAP } }
  }

  function closeLimit() {
    limitTarget = null
  }

  async function saveLimit() {
    const target = limitTarget
    const teacherCount = limitValue
    if (!target) return
    if (teacherCount === null || !Number.isInteger(teacherCount) || teacherCount < 1) {
      limitHint = $t('outcome.teacher_count_invalid')
      return
    }
    limitHint = ''
    await once(`row|${slotKey(target.day, target.slot)}`, async () => {
      const outcome = await request('SetSlotTeacherCount', () => App.SetSlotTeacherCount({
        semesterId,
        day: target.day,
        slot: target.slot,
        teacherCount
      }))
      if (outcome && checkOutcome(outcome)) {
        limitTarget = null
        await refresh()
      }
    })
  }

  // ----- Очистка назначений и удаление строки -----

  let clearTarget = $state<{ day: number; slot: string; assignments: number } | null>(null)
  let deleteTarget = $state<{ day: number; slot: string; pairs: number; assignments: number } | null>(null)

  function askClear(day: number, slot: SlotView) {
    clearTarget = { day, slot: slot.slot, assignments: slot.assignmentCount }
  }

  function askDelete(day: number, slot: SlotView) {
    deleteTarget = { day, slot: slot.slot, pairs: slot.pairCount, assignments: slot.assignmentCount }
  }

  async function confirmClear() {
    const target = clearTarget
    if (!target) return
    await once(`row|${slotKey(target.day, target.slot)}`, async () => {
      if (await requestFlag('ClearSlotAssignments', () => App.ClearSlotAssignments({ semesterId, day: target.day, slot: target.slot }))) {
        await refresh()
      }
      clearTarget = null
    })
  }

  async function confirmDelete() {
    const target = deleteTarget
    if (!target) return
    await once(`row|${slotKey(target.day, target.slot)}`, async () => {
      if (await requestFlag('DeleteSlot', () => App.DeleteSlot({ semesterId, day: target.day, slot: target.slot }))) {
        await refresh()
      }
      deleteTarget = null
    })
  }

  // ----- Экспорт -----

  async function exportXLSX() {
    if (!semesterId) return
    try {
      const fileName = await request('GetExportFileName', () => App.GetExportFileName(semesterId))
      if (fileName === null) return
      const path = await save({
        title: $t('export_xlsx_title'),
        defaultPath: fileName,
        filters: [{ name: $t('save_xlsx_filter'), extensions: ['xlsx'] }]
      })
      if (!path) return
      const exportedPath = await request('ExportScheduleXLSX', () => App.ExportScheduleXLSX(semesterId, path))
      if (exportedPath === null) return
      notify($t('export_done', { values: { path: exportedPath } }), 'info')
    } catch (e) {
      console.error('Export failed:', e)
      notify($t('export_failed'))
    }
  }

  // ----- Привязка преподавателей -----

  let manageOpen = $state(false)
  let teacherSearch = $state('')
  let unbindTarget = $state<{ teacher: Teacher; result: UnbindResult } | null>(null)

  function openManage() {
    manageOpen = true
    loadSemesterTeachers(semesterId)
  }

  function closeManage() {
    manageOpen = false
  }

  const filteredSemesterTeachers = $derived.by(() => {
    const q = teacherSearch.trim().toLowerCase()
    const list = $semesterTeachersStore
    if (!q) return list
    return list.filter(t => `${t.lastName} ${t.firstName} ${t.middleName}`.toLowerCase().includes(q))
  })

  // Снятие привязки сначала показывает, сколько назначений и пар уйдёт; выполняется после подтверждения.
  async function toggleBind(teacher: Teacher) {
    await once(`bind|${teacher.id}`, async () => {
      if (teacher.isBound) {
        const result = await previewUnbindTeacher(semesterId, teacher.id)
        if (result) unbindTarget = { teacher, result }
        return
      }
      if (await bindTeacherToSemester(semesterId, teacher.id)) await refresh()
    })
  }

  async function confirmUnbind() {
    const target = unbindTarget
    if (!target) return
    await once('unbind', async () => {
      const result = await unbindTeacherFromSemester(semesterId, target.teacher.id)
      unbindTarget = null
      if (!result) return
      notify($t('unbind_done', { values: { assignments: result.assignments } }), 'info')
      await refresh()
    })
  }

  // ----- Ставка -----

  const rateBarHeight = (t: TeacherView): number =>
    Math.min(100, Math.round((t.assigned / Math.max(1, t.target)) * 100))

  function rateColor(t: TeacherView): string {
    const remaining = t.target - t.assigned
    if (remaining === 0) return 'bg-emerald-400'
    if (remaining < 0) return 'bg-amber-400'
    return 'bg-indigo-500'
  }

  const cellClass = (assigned: boolean): string => `cell ${assigned ? 'cell-on' : 'cell-off'}`

  // Фиксированная ширина колонок преподавателей: не зависит от размера окна, иначе таблица перераскладывается при каждом обновлении.
  const TEACHER_COL_PX = 120

  // Липкие колонки слева (день, время, факультеты, количество): видны при горизонтальной прокрутке.
  // Факультеты — последняя липкая колонка и по содержимому, поэтому её ширину не задаём.
  const FROZEN_WIDTH = { day: '3rem', time: '6rem', cnt: '7rem' } as const
  const FROZEN_LEFT = {
    day: '0px',
    time: 'var(--col-day)',
    cnt: 'calc(var(--col-day) + var(--col-time))',
    fac: 'calc(var(--col-day) + var(--col-time) + var(--col-cnt))',
  } as const
  const frozenStyle = (col: keyof typeof FROZEN_LEFT): string => {
    const left = `left: ${FROZEN_LEFT[col]}`
    if (col === 'fac') return left
    return `${left}; width: var(--col-${col}); min-width: var(--col-${col}); max-width: var(--col-${col})`
  }
</script>

<div class="w-full h-screen flex flex-col px-4 py-4">
  <div class="mb-4 flex items-center justify-between">
    <div class="flex items-center gap-3">
      <button class="back-link" onclick={back}>← {$t('back')}</button>
      {#if semester}
        <h2 class="text-2xl font-semibold">{semester.name}</h2>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      <button class="rounded-md bg-indigo-600 hover:bg-indigo-500 text-white px-3 py-2 text-xs" onclick={openAddSlot}>{$t('add_slot')}</button>
      <button class="rounded-md bg-purple-600 hover:bg-purple-500 text-white px-3 py-2 text-xs" onclick={openManage}>{$t('assign_teachers')}</button>
      <button class="rounded-md bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white px-3 py-2 text-xs" onclick={exportXLSX}>{$t('export_xlsx')}</button>
    </div>
  </div>

  {#if loadError}
    <div class="text-sm text-rose-600">{loadError}</div>
  {/if}

  {#if addSlotPopover}
    <button class="fixed inset-0 z-30 cursor-default" aria-label={$t('close')} tabindex="-1" onclick={closeAddSlot}></button>
    <div class="popover" style={`left:${addSlotPopover.x}px; top:${addSlotPopover.y}px`} role="dialog" aria-modal="true" tabindex="-1" use:overlay={{ onclose: closeAddSlot }}>
      <div class="text-sm text-slate-700 mb-2">{$t('add_slot')}</div>
      <div class="grid gap-3 text-sm">
        <div class="flex items-center gap-2">
          <div class="text-slate-500 w-24">{$t('select_day')}</div>
          <div class="flex flex-wrap items-center gap-1 min-w-0">
            {#each days as d (d.key)}
              <button class:chip-active={addSlotDay === d.key} class="chip" onclick={() => setAddSlotDay(d.key)}>{d.label}</button>
            {/each}
          </div>
        </div>
        {#if addSlotCustom}
          <div class="grid grid-cols-[max-content_1fr_max-content] grid-rows-2 gap-x-4 items-center min-w-0">
            <div class="text-slate-500 text-xs col-start-1 row-start-1">{$t('start_time')}</div>
            <div class="text-slate-500 text-xs col-start-3 row-start-1">{$t('end_time')}</div>
            <input type="time" class="w-24 rounded bg-white px-2 py-1 ring-1 ring-slate-300 col-start-1 row-start-2" bind:value={addSlotStart} />
            <div class="text-slate-400 text-center col-start-2 row-start-2 justify-self-center">—</div>
            <input type="time" class="w-24 rounded bg-white px-2 py-1 ring-1 ring-slate-300 col-start-3 row-start-2" bind:value={addSlotEnd} />
          </div>
        {:else if freePairs.length > 0}
          <div class="flex flex-wrap gap-1">
            {#each freePairs as p (pairKey(p))}
              <button class:chip-active={addSlotPairKey === pairKey(p)} class="chip" onclick={() => (addSlotPairKey = pairKey(p))}>{pairKey(p)}</button>
            {/each}
          </div>
        {:else}
          <div class="muted text-xs">{$t('no_free_pairs')}</div>
        {/if}
        <div>
          <button class="chip" onclick={() => (addSlotCustom = !addSlotCustom)}>{addSlotCustom ? $t('standard_time') : $t('custom_time')}</button>
        </div>
        <div class="flex items-center justify-between gap-2">
          <div class="text-slate-500 text-xs">{$t('teacher_count')}</div>
          <input type="number" min="1" step="1" class="w-24 rounded bg-white px-2 py-1 ring-1 ring-slate-300" bind:value={addSlotTeacherCount} placeholder={$t('empty')} />
        </div>
        {#if addSlotHint}
          <div class="text-xs text-rose-600">{addSlotHint}</div>
        {/if}
        <div class="flex items-center justify-end gap-2 mt-1">
          <button class="rounded-md bg-slate-200 hover:bg-slate-300 px-3 py-1.5 text-xs" onclick={closeAddSlot}>{$t('cancel')}</button>
          <button class="rounded-md bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 text-white px-3 py-1.5 text-xs" onclick={createTimeSlot}>{$t('create')}</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="flex-1 overflow-auto custom-scroll" style={`--teacher-col-width:${TEACHER_COL_PX}px; --col-day:${FROZEN_WIDTH.day}; --col-time:${FROZEN_WIDTH.time}; --col-cnt:${FROZEN_WIDTH.cnt}`}>
    {#if view}
      <table class="w-max border-separate text-sm" style="border-spacing: 0;">
        <colgroup>
          <col style="width: var(--col-day)" />
          <col style="width: var(--col-time)" />
          <col style="width: var(--col-cnt)" />
          <col />
          {#each view.teachers as _, i (i)}
            <col style="width: var(--teacher-col-width)" />
          {/each}
          <col />
        </colgroup>
        <thead>
          <tr>
            <th style={frozenStyle('day')} class="sticky z-20 bg-white text-center p-2 border-b border-slate-200">{$t('day')}</th>
            <th style={frozenStyle('time')} class="sticky z-20 bg-white text-center p-2 border-b border-slate-200">{$t('time')}</th>
            <th style={frozenStyle('cnt')} class="sticky z-20 bg-white p-2 border-b border-slate-200 text-center">
              <div class="text-xs leading-tight">{$t('teacher_count')}</div>
            </th>
            <th style={frozenStyle('fac')} class="sticky z-20 bg-white p-2 border-b border-slate-200 text-left whitespace-nowrap">{$t('faculty')}</th>
            {#each view.teachers as teacher (teacher.id)}
              <th class="p-2 border-b border-slate-200 text-left align-bottom" style="width: var(--teacher-col-width); min-width: var(--teacher-col-width); max-width: var(--teacher-col-width);">
                <div class="text-xs text-slate-500">{teacher.directionName}{#if teacher.directionArchived} {$t('archived_suffix')}{/if}</div>
                <div class="flex items-start gap-1">
                  <div class="min-w-0 flex-1">
                    <div class="text-sm font-medium leading-tight" title={formatTeacherName(teacher)}>{formatTeacherName(teacher)}</div>
                    <div class="mt-1 text-[10px] text-slate-500">{teacher.assigned}/{teacher.target}</div>
                  </div>
                  <div class="flex h-12 w-1.5 shrink-0 flex-col justify-end overflow-hidden rounded bg-slate-300">
                    <div class={rateColor(teacher)} style={`height: ${rateBarHeight(teacher)}%`}></div>
                  </div>
                </div>
              </th>
            {/each}
            <th class="p-2 border-b border-slate-200 text-right"></th>
          </tr>
        </thead>
        <tbody>
          {#each dayRows as d (d.key)}
            {#each d.rows as row, i (row.slot.slot)}
              <tr>
                {#if i === 0}
                  <td style={frozenStyle('day')} class="sticky z-10 bg-white p-2 border-b border-slate-200 align-top text-center" rowspan={d.rows.length}>{d.label}</td>
                {/if}
                <td style={frozenStyle('time')} class="sticky z-10 bg-white p-2 border-b border-slate-200 font-medium text-center">{row.slot.startTime}-{row.slot.endTime}</td>
                <td style={frozenStyle('cnt')} class="sticky z-10 bg-white p-2 border-b border-slate-200 text-center align-middle">
                  <button
                    class="text-xs text-slate-600 hover:underline disabled:no-underline"
                    disabled={row.slot.teacherMax === null}
                    onclick={(e) => openLimit(e, d.key, row.slot)}
                    title={$t('teacher_count')}
                  >
                    {row.slot.teacherMax !== null ? `${row.slot.teacherCurrent}/${row.slot.teacherMax}` : ''}
                  </button>
                </td>
                <td style={frozenStyle('fac')} class="sticky z-10 bg-white p-2 border-b border-slate-200 whitespace-nowrap">
                  <div class="inline-flex flex-nowrap gap-1">
                    {#each row.slot.facultyIds as fid (fid)}
                      {@const faculty = facultyById.get(fid)}
                      <span class="px-2 py-1 text-[10px] rounded bg-slate-200 truncate max-w-[80px]" title={faculty?.name ?? ''}>{faculty?.name ?? ''}{#if faculty?.isArchived} {$t('archived_suffix')}{/if}</span>
                    {/each}
                    <button class="tag" onclick={(e) => openFacultyPopover(e, d.key, row.slot.slot)}>+</button>
                  </div>
                </td>
                {#each view.teachers as teacher (teacher.id)}
                  <td class="p-2 border-b border-slate-200 align-middle overflow-hidden" style="width: var(--teacher-col-width); min-width: var(--teacher-col-width); max-width: var(--teacher-col-width);">
                    <button
                      class={cellClass(row.assigned.has(teacher.id))}
                      onclick={() => toggleTeacher(d.key, row.slot.slot, teacher.id)}
                      title={row.assigned.has(teacher.id) ? $t('unassign_teacher') : $t('assign_teacher')}
                      aria-label={row.assigned.has(teacher.id) ? $t('unassign_teacher') : $t('assign_teacher')}
                    >
                      +
                    </button>
                  </td>
                {/each}

                <td class="p-2 border-b border-slate-200 text-right align-middle whitespace-nowrap">
                  <div class="inline-flex items-center gap-1">
                    <button
                      class="px-2 py-1 text-xs rounded bg-slate-200 hover:bg-slate-300 disabled:opacity-50 disabled:hover:bg-slate-200"
                      disabled={row.slot.assignmentCount === 0}
                      onclick={() => askClear(d.key, row.slot)}
                    >{$t('clear_assignments')}</button>
                    <button class="px-2 py-1 text-xs rounded bg-rose-600/90 hover:bg-rose-600 text-white" title={$t('delete')} aria-label={$t('delete')} onclick={() => askDelete(d.key, row.slot)}>×</button>
                  </div>
                </td>
              </tr>
            {/each}
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  {#if facultyPopover && view}
    <button class="fixed inset-0 z-30 cursor-default" aria-label={$t('close')} tabindex="-1" onclick={closeFacultyPopover}></button>
    <div class="popover max-h-[80vh] overflow-auto p-3" style={`left:${facultyPopover.pos.x}px; top:${facultyPopover.pos.y}px`} role="dialog" aria-modal="true" tabindex="-1" use:overlay={{ onclose: closeFacultyPopover }}>
      <div class="text-xs text-slate-600 mb-2">{$t('add_faculty')}</div>
      <div class="max-h-64 overflow-auto grid gap-1 mb-3">
        {#each popoverFaculties as f (f.id)}
          <label class="flex items-center gap-2 text-sm">
            <input type="checkbox" class="accent-indigo-600" checked={f.checked} onchange={() => toggleRowFaculty(f.id)} />
            {f.name}{#if f.isArchived} {$t('archived_suffix')}{/if}
          </label>
        {/each}
      </div>
      <div class="flex items-center justify-end gap-2">
        <button class="rounded-md bg-slate-200 hover:bg-slate-300 px-3 py-1.5 text-xs" onclick={closeFacultyPopover}>{$t('close')}</button>
      </div>
    </div>
  {/if}

  {#if limitTarget}
    <button class="fixed inset-0 z-30 cursor-default" aria-label={$t('close')} tabindex="-1" onclick={closeLimit}></button>
    <div class="popover p-3" style={`left:${limitTarget.pos.x}px; top:${limitTarget.pos.y}px`} role="dialog" aria-modal="true" tabindex="-1" use:overlay={{ onclose: closeLimit }}>
      <div class="flex items-center justify-between gap-2 text-sm">
        <div class="text-slate-500 text-xs">{$t('teacher_count')}</div>
        <input type="number" min="1" step="1" class="w-24 rounded bg-white px-2 py-1 ring-1 ring-slate-300" bind:value={limitValue} />
      </div>
      {#if limitHint}
        <div class="mt-2 text-xs text-rose-600">{limitHint}</div>
      {/if}
      <div class="mt-3 flex items-center justify-end gap-2">
        <button class="rounded-md bg-slate-200 hover:bg-slate-300 px-3 py-1.5 text-xs" onclick={closeLimit}>{$t('cancel')}</button>
        <button class="rounded-md bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 text-white px-3 py-1.5 text-xs" onclick={saveLimit}>{$t('save')}</button>
      </div>
    </div>
  {/if}

  {#if manageOpen}
    <div class="fixed inset-0 z-30 flex items-center justify-center">
      <button class="absolute inset-0 bg-black/40 cursor-default" aria-label={$t('close')} tabindex="-1" onclick={closeManage}></button>
      <div class="panel relative max-h-[80vh] w-[720px] max-w-[92vw] overflow-auto p-4" role="dialog" aria-modal="true" tabindex="-1" use:overlay={{ onclose: closeManage }}>
        <div class="flex items-center justify-between mb-3">
          <div class="text-sm font-medium">{$t('assign_teachers')}</div>
          <button class="px-2 py-1 text-xs rounded bg-slate-200 hover:bg-slate-300" onclick={closeManage}>{$t('close')}</button>
        </div>
        <div class="mb-2">
          <input class="w-full rounded bg-white px-2 py-1 ring-1 ring-slate-300 text-sm" placeholder={$t('search_placeholder')} bind:value={teacherSearch} />
        </div>
        <div class="grid grid-cols-1 gap-1">
          {#each filteredSemesterTeachers as st (st.id)}
            <div class="flex items-center justify-between px-2 py-2 rounded ring-1 ring-black/10">
              <div class="min-w-0">
                <div class="text-sm font-medium truncate" title={`${st.lastName} ${st.firstName} ${st.middleName}`}>{st.lastName} {st.firstName} {st.middleName}</div>
                <div class="text-xs text-slate-500 flex items-center gap-2">
                  <span>{st.directionName}{#if st.directionArchived} {$t('archived_suffix')}{/if}</span>
                  {#if st.isGuest}<span class="px-1 rounded bg-amber-100 text-amber-800">{$t('guest_marker')}</span>{/if}
                  {#if st.isArchived}<span class="px-1 rounded bg-slate-200 text-slate-800">{$t('archived_marker')}</span>{/if}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button class="px-2 py-1 text-xs rounded {st.isBound ? 'bg-rose-600/90 hover:bg-rose-600 text-white' : 'bg-indigo-600 hover:bg-indigo-500 text-white'}" onclick={() => toggleBind(st)}>
                  {st.isBound ? $t('unbind') : $t('add_teacher')}
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  <ConfirmModal
    open={deleteTarget !== null}
    title={$t('confirm_title')}
    message={deleteTarget ? $t('delete_slot_message', { values: { pairs: deleteTarget.pairs, assignments: deleteTarget.assignments } }) : ''}
    confirmText={$t('delete')}
    cancelText={$t('cancel')}
    onconfirm={confirmDelete}
    oncancel={() => (deleteTarget = null)}
  />
  <ConfirmModal
    open={clearTarget !== null}
    title={$t('confirm_title')}
    message={clearTarget ? $t('clear_assignments_message', { values: { count: clearTarget.assignments } }) : ''}
    confirmText={$t('clear_assignments')}
    cancelText={$t('cancel')}
    onconfirm={confirmClear}
    oncancel={() => (clearTarget = null)}
  />
  <ConfirmModal
    open={unbindTarget !== null}
    title={$t('confirm_title')}
    message={unbindTarget ? $t('unbind_message', { values: { assignments: unbindTarget.result.assignments } }) : ''}
    confirmText={$t('unbind')}
    cancelText={$t('cancel')}
    onconfirm={confirmUnbind}
    oncancel={() => (unbindTarget = null)}
  />
</div>
