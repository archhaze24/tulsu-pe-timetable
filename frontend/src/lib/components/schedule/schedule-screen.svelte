<script lang="ts">
  import { t } from 'svelte-i18n'
  import { onMount } from 'svelte'
  import { save } from '@tauri-apps/plugin-dialog'
  import * as App from '../../api'
  import type { ScheduleView, SlotView, ScheduleOutcome, ScheduleTeacher, SemesterTeacher } from '../../api'
  import { navigate, route } from '../../stores/router'
  import { semestersStore, loadSemesters } from '../../stores/semesters'
  import { formatTeacherName } from '../../stores/teachers'
  import { request } from '../../stores/request'
  import { notify } from '../../stores/notify'
  import { loadSemesterTeachers, bindTeacherToSemester, unbindTeacherFromSemester, semesterTeachersStore } from '../../stores/semester-teachers'

  const semesterId = $derived(Number($route.params?.id ?? 0) || $semestersStore[0]?.id || 0)
  const semester = $derived($semestersStore.find(s => s.id === semesterId))


  let view = $state<ScheduleView | null>(null)
  let loadError = $state('')

  const back = () => { navigate('semesters') }

  async function reload(id: number) {
    const resp = await App.GetSchedule(id)
    if (id !== semesterId) return
    if (resp.error) {
      loadError = resp.error
      return
    }
    loadError = ''
    view = resp.data
  }

  const refresh = () => reload(semesterId)

  onMount(() => { loadSemesters() })

  $effect(() => {
    const id = semesterId
    if (!id) return
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

  // Бэкенд возвращает коды; тексты берём из локалей фронтенда.
  function outcomeMessage(outcome: ScheduleOutcome): string | null {
    switch (outcome.code) {
      case 'ok': return null
      case 'teacher_count_exceeded': return $t('teacher_count_limit_reached').replace('{count}', String(outcome.max))
      case 'teacher_count_required': return $t('teacher_count_required')
      case 'slot_exists': return $t('slot_exists')
      case 'select_direction': return $t('select_direction')
    }
  }

  function reportOutcome(outcome: ScheduleOutcome) {
    const message = outcomeMessage(outcome)
    if (message) notify(message)
  }

  const slotOf = (day: number, slot: string): SlotView | undefined =>
    view?.days.find(d => d.day === day)?.slots.find(s => s.slot === slot)

  const slotsOf = (day: number): SlotView[] => view?.days.find(d => d.day === day)?.slots ?? []

  const facultyName = (id: number): string => view?.faculties.find(f => f.id === id)?.name ?? ''

  const isAssigned = (slot: SlotView, teacherId: number): boolean => slot.assignedTeacherIds.includes(teacherId)

  async function toggleTeacher(day: number, slot: string, teacherId: number) {
    const outcome = await request('ToggleTeacherInSlot', () => App.ToggleTeacherInSlot({ semesterId, day, slot, teacherId }))
    if (!outcome) return
    reportOutcome(outcome)
    await refresh()
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
  let addSlotStart = $state('09:40')
  let addSlotEnd = $state('11:15')
  let addSlotTeacherCount = $state<number | undefined>(undefined)

  function openAddSlot(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    addSlotPopover = { x: popoverLeft(rect), y: rect.bottom + EDGE_GAP }
  }

  function closeAddSlot() {
    addSlotPopover = null
  }

  async function createTimeSlot() {
    const teacherCount = addSlotTeacherCount
    if (!semesterId) return
    if (!teacherCount || teacherCount < 1) {
      notify($t('teacher_count_required'))
      return
    }
    const outcome = await request('AddSlot', () => App.AddSlot({
      semesterId,
      day: addSlotDay,
      startTime: addSlotStart,
      endTime: addSlotEnd,
      teacherCount
    }))
    if (!outcome) return
    if (outcome.code !== 'ok') {
      reportOutcome(outcome)
      return
    }
    addSlotPopover = null
    await refresh()
  }

  // ----- Факультеты строки -----

  let facultyPopover = $state<{ day: number; slot: string; pos: Point } | null>(null)

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

  async function toggleRowFaculty(day: number, slot: string, facultyId: number) {
    const current = slotOf(day, slot)?.facultyIds ?? []
    const next = current.includes(facultyId) ? current.filter(id => id !== facultyId) : [...current, facultyId]
    const ok = await request('SetSlotFaculties', () => App.SetSlotFaculties({ semesterId, day, slot, facultyIds: next }))
    if (ok === undefined) return
    await refresh()
  }

  // ----- Удаление строки -----

  async function deleteRow(day: number, slot: string) {
    const ok = await request('DeleteSlot', () => App.DeleteSlot({ semesterId, day, slot }))
    if (ok === undefined) return
    await refresh()
  }

  // ----- Экспорт -----

  async function exportXLSX() {
    if (!semesterId) return
    try {
      const fileName = await request('GetExportFileName', () => App.GetExportFileName(semesterId))
      if (fileName === undefined) return
      const path = await save({
        title: $t('export_xlsx_title'),
        defaultPath: fileName,
        filters: [{ name: $t('save_xlsx_filter'), extensions: ['xlsx'] }]
      })
      if (!path) {
        return
      }
      const exportedPath = await request('ExportScheduleXLSX', () => App.ExportScheduleXLSX(semesterId, path))
      if (exportedPath === undefined) return
    } catch (e) {
      notify($t('export_failed'))
    }
  }

  // ----- Привязка преподавателей -----

  let manageOpen = $state(false)
  let teacherSearch = $state('')

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
    return list.filter(t => `${t.lastName} ${t.firstName} ${t.middleName ?? ''}`.toLowerCase().includes(q))
  })

  async function toggleBind(st: SemesterTeacher) {
    if (st.isBound) {
      await unbindTeacherFromSemester(semesterId, st.id)
    } else {
      const ok = await bindTeacherToSemester(semesterId, st.id)
      if (!ok) notify($t('guest_only_one_semester'))
    }
    await refresh()
  }

  // ----- Ставка -----

  const rateBarHeight = (t: ScheduleTeacher): number =>
    Math.min(100, Math.round((t.assigned / Math.max(1, t.target)) * 100))

  function rateColor(t: ScheduleTeacher): string {
    const remaining = t.target - t.assigned
    if (remaining === 0) return 'bg-emerald-400'
    if (remaining < 0) return 'bg-amber-400'
    return 'bg-indigo-500'
  }

  const cellClass = (assigned: boolean): string => `cell ${assigned ? 'cell-on' : 'cell-off'}`

  // Фиксированная ширина колонок преподавателей: не зависит от размера окна, иначе таблица перераскладывается при каждом обновлении.
  const TEACHER_COL_PX = 120
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
      <button class="rounded-md bg-indigo-600 hover:bg-indigo-500 text-white px-3 py-2 text-xs" onclick={openAddSlot} data-log-action="add_slot">{$t('add_slot')}</button>
      <button class="rounded-md bg-purple-600 hover:bg-purple-500 text-white px-3 py-2 text-xs" onclick={openManage} data-log-action="assign_teachers">{$t('assign_teachers')}</button>
      <button class="rounded-md bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white px-3 py-2 text-xs" onclick={exportXLSX} data-log-action="export_xlsx">{$t('export_xlsx')}</button>
    </div>
  </div>

  {#if loadError}
    <div class="text-sm text-rose-600">{loadError}</div>
  {/if}

  {#if addSlotPopover}
    <div class="popover" style={`left:${addSlotPopover.x}px; top:${addSlotPopover.y}px`} role="dialog" aria-modal="true">
      <div class="text-sm text-slate-700 dark:text-slate-300 mb-2">{$t('add_slot')}</div>
      <div class="grid gap-3 text-sm">
        <div class="flex items-center gap-2">
          <div class="text-slate-500 dark:text-slate-400 w-24">{$t('select_day')}</div>
          <div class="flex flex-wrap items-center gap-1 min-w-0">
            {#each days as d (d.key)}
              <button class:chip-active={addSlotDay === d.key} class="chip" onclick={() => (addSlotDay = d.key)}>{d.label}</button>
            {/each}
          </div>
        </div>
        <div class="grid grid-cols-[max-content_1fr_max-content] grid-rows-2 gap-x-4 items-center min-w-0">
          <div class="text-slate-500 dark:text-slate-400 text-xs col-start-1 row-start-1">{$t('start_time')}</div>
          <div class="text-slate-500 dark:text-slate-400 text-xs col-start-3 row-start-1">{$t('end_time')}</div>
          <input class="w-24 rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10 col-start-1 row-start-2" bind:value={addSlotStart} />
          <div class="text-slate-400 text-center col-start-2 row-start-2 justify-self-center">—</div>
          <input class="w-24 rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10 col-start-3 row-start-2" bind:value={addSlotEnd} />
        </div>
        <div class="flex items-center justify-between gap-2">
          <div class="text-slate-500 dark:text-slate-400 text-xs">{$t('teacher_count')}</div>
          <input type="number" min="1" class="w-24 rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10" bind:value={addSlotTeacherCount} placeholder={$t('empty')} />
        </div>
        <div class="flex items-center justify-end gap-2 mt-1">
          <button class="rounded-md bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600 px-3 py-1.5 text-xs" onclick={closeAddSlot}>{$t('cancel')}</button>
          <button class="rounded-md bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 text-white px-3 py-1.5 text-xs" onclick={createTimeSlot}>{$t('create')}</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="flex-1 overflow-auto custom-scroll" style={`--teacher-col-width:${TEACHER_COL_PX}px`}>
    {#if view}
      <table class="w-max border-separate text-sm" style="border-spacing: 0;">
        <colgroup>
          <col style="width: 3rem" />
          <col style="width: 6rem" />
          <col style="width: 1%" />
          {#each view.teachers as _, i (i)}
            <col style="width: var(--teacher-col-width)" />
          {/each}
          <col style="width: 4rem" />
          <col style="width: 3.5rem" />
        </colgroup>
        <thead>
          <tr>
            <th class="sticky left-0 z-20 bg-white dark:bg-slate-900 text-center p-2 border-b border-slate-200 dark:border-slate-700 w-12">{$t('day')}</th>
            <th class="sticky left-0 z-20 bg-white dark:bg-slate-900 text-center p-2 border-b border-slate-200 dark:border-slate-700 w-24">{$t('time')}</th>
            <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-left whitespace-nowrap">{$t('faculty')}</th>
            {#each view.teachers as teacher (teacher.id)}
              <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-left align-bottom" style="width: var(--teacher-col-width); min-width: var(--teacher-col-width); max-width: var(--teacher-col-width);">
                <div class="text-xs text-slate-500 dark:text-slate-400">{teacher.directionName}</div>
                <div class="flex items-start gap-1">
                  <div class="min-w-0 flex-1">
                    <div class="text-sm font-medium leading-tight" title={formatTeacherName(teacher)}>{formatTeacherName(teacher)}</div>
                    <div class="mt-1 text-[10px] text-slate-500 dark:text-slate-400">{teacher.assigned}/{teacher.target}</div>
                  </div>
                  <div class="flex h-12 w-1.5 shrink-0 flex-col justify-end overflow-hidden rounded bg-slate-300 dark:bg-slate-700">
                    <div class={rateColor(teacher)} style={`height: ${rateBarHeight(teacher)}%`}></div>
                  </div>
                </div>
              </th>
            {/each}
            <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-center whitespace-normal">
              <div class="text-xs leading-tight">{$t('teacher_count')}</div>
            </th>
            <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-right w-14"></th>
          </tr>
        </thead>
        <tbody>
          {#each days as d (d.key)}
            {#each slotsOf(d.key) as slot, i (slot.slot)}
              <tr>
                {#if i === 0}
                  <td class="sticky left-0 z-10 bg-white dark:bg-slate-900 p-2 border-b border-slate-200 dark:border-slate-800 align-top text-center w-12" rowspan={slotsOf(d.key).length}>{d.label}</td>
                {/if}
                <td class="sticky left-0 z-10 bg-white dark:bg-slate-900 p-2 border-b border-slate-200 dark:border-slate-800 font-medium text-center w-24">{slot.startTime}-{slot.endTime}</td>
                <td class="p-2 border-b border-slate-200 dark:border-slate-800 whitespace-nowrap">
                  <div class="inline-flex flex-nowrap gap-1">
                    {#each slot.facultyIds as fid (fid)}
                      <span class="px-2 py-1 text-[10px] rounded bg-slate-200 dark:bg-slate-700 truncate max-w-[80px]" title={facultyName(fid)}>{facultyName(fid)}</span>
                    {/each}
                    <button class="tag" onclick={(e) => openFacultyPopover(e, d.key, slot.slot)} data-log-action="open_faculty_popover">+</button>
                  </div>
                </td>
                {#each view.teachers as teacher (teacher.id)}
                  <td class="p-2 border-b border-slate-200 dark:border-slate-800 align-middle overflow-hidden" style="width: var(--teacher-col-width); min-width: var(--teacher-col-width); max-width: var(--teacher-col-width);">
                    <button
                      class={cellClass(isAssigned(slot, teacher.id))}
                      onclick={() => toggleTeacher(d.key, slot.slot, teacher.id)}
                      title={isAssigned(slot, teacher.id) ? $t('unassign_teacher') : $t('assign_teacher')}
                      aria-label={isAssigned(slot, teacher.id) ? $t('unassign_teacher') : $t('assign_teacher')}
                      data-log-action={isAssigned(slot, teacher.id) ? 'unassign_teacher' : 'assign_teacher'}
                      data-teacher-id={teacher.id}
                      data-day={d.key}
                      data-slot={slot.slot}
                    >
                      +
                    </button>
                  </td>
                {/each}
                <td class="p-2 border-b border-slate-200 dark:border-slate-800 text-center align-middle">
                  <div class="text-xs text-slate-600 dark:text-slate-400">
                    {slot.teacherMax !== null ? `${slot.teacherCurrent}/${slot.teacherMax}` : ''}
                  </div>
                </td>
                <td class="p-2 border-b border-slate-200 dark:border-slate-800 text-right align-middle">
                  <button class="px-2 py-1 text-xs rounded bg-rose-600/90 hover:bg-rose-600 text-white" title={$t('delete')} aria-label={$t('delete')} onclick={() => deleteRow(d.key, slot.slot)} data-log-action="delete_time_slot">×</button>
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
    <div class="popover max-h-[80vh] overflow-auto p-3" style={`left:${facultyPopover.pos.x}px; top:${facultyPopover.pos.y}px`} role="dialog" aria-modal="true">
      <div class="text-xs text-slate-600 dark:text-slate-400 mb-2">{$t('add_faculty')}</div>
      <div class="max-h-64 overflow-auto grid gap-1 mb-3">
        {#each view.faculties as f (f.id)}
          <label class="flex items-center gap-2 text-sm">
            <input type="checkbox" class="accent-indigo-600" checked={slotOf(facultyPopover.day, facultyPopover.slot)?.facultyIds.includes(f.id) ?? false} onchange={() => toggleRowFaculty(facultyPopover!.day, facultyPopover!.slot, f.id)} />
            {f.name}
          </label>
        {/each}
      </div>
      <div class="flex items-center justify-end gap-2">
        <button class="rounded-md bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600 px-3 py-1.5 text-xs" onclick={closeFacultyPopover}>{$t('close')}</button>
      </div>
    </div>
  {/if}

  {#if manageOpen}
    <div class
="fixed inset-0 z-30 flex items-center justify-center">
      <button class="absolute inset-0 bg-black/40 cursor-default" aria-label={$t('close')} tabindex="-1" onclick={closeManage}></button>
      <div class="panel relative max-h-[80vh] w-[720px] max-w-[92vw] overflow-auto p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="text-sm font-medium">{$t('assign_teachers')}</div>
          <button class="px-2 py-1 text-xs rounded bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600" onclick={closeManage} data-log-action="close_manage_teachers">{$t('close')}</button>
        </div>
        <div class="mb-2">
          <input class="w-full rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10 text-sm" placeholder={$t('search_placeholder')} bind:value={teacherSearch} />
        </div>
        <div class="grid grid-cols-1 gap-1">
          {#each filteredSemesterTeachers as st (st.id)}
            <div class="flex items-center justify-between px-2 py-2 rounded ring-1 ring-black/10 dark:ring-white/10">
              <div class="min-w-0">
                <div class="text-sm font-medium truncate" title={`${st.lastName} ${st.firstName} ${st.middleName ?? ''}`}>{st.lastName} {st.firstName} {st.middleName}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400 flex items-center gap-2">
                  <span>{st.directionName || ''}</span>
                  {#if st.isGuest}<span class="px-1 rounded bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">{$t('guest_marker')}</span>{/if}
                  {#if st.isArchived}<span class="px-1 rounded bg-slate-200 text-slate-800 dark:bg-slate-800 dark:text-slate-200">{$t('archived_marker')}</span>{/if}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button class="px-2 py-1 text-xs rounded {st.isBound ? 'bg-rose-600/90 hover:bg-rose-600 text-white' : 'bg-indigo-600 hover:bg-indigo-500 text-white'}" onclick={() => toggleBind(st)} data-log-action={st.isBound ? 'unbind_teacher' : 'bind_teacher'} data-teacher-id={st.id}>
                  {st.isBound ? $t('unbind') : $t('add_teacher')}
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}


</div>
