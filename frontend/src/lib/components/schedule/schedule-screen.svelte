<script lang="ts">
  import { t } from 'svelte-i18n'
  import { facultiesStore, refreshFaculties, fetchArchivedFaculties, type Faculty } from '../../stores/faculties'
  import { directionsStore } from '../../stores/directions'
  import { teachersStore, formatTeacherName } from '../../stores/teachers'
  import { semesterTeachersStore, loadSemesterTeachers, bindTeacherToSemester, unbindTeacherFromSemester, type SemesterTeacher } from '../../stores/semester-teachers'
  import { lessonsStore, semestersStore, addLesson, deleteLesson, updateLesson, loadLessonsBySemester, loadSemesters, type Lesson } from '../../stores/semesters'
  import { navigate, route } from '../../stores/router'
  import { onDestroy, onMount } from 'svelte'
  import { get } from 'svelte/store'
  import { refreshDirections } from '../../stores/directions'
  
  import { logUiEvent } from '../../stores/logger'
  import * as App from '../../../../wailsjs/go/app_services/App.js'

  const back = () => { logUiEvent('click:back', { screen: 'schedule', semesterId }); navigate('semesters') }

  $: semesterId = Number($route.params?.id ?? 0) || $semestersStore[0]?.id || 0
  $: if (semesterId) { loadSemesterTeachers(semesterId) }
  $: if (semesterId) { loadLessonsBySemester(semesterId) }
  $: if (!$semestersStore || $semestersStore.length === 0) { loadSemesters() }
  $: if (!$directionsStore || $directionsStore.length === 0) { refreshDirections() }
  $: if (!$facultiesStore || $facultiesStore.length === 0) { refreshFaculties() }
  $: semester = $semestersStore.find(s => s.id === semesterId)

  // Include archived faculties for display of already assigned items
  let archivedFaculties: Faculty[] = []
  onMount(async () => {
    archivedFaculties = await fetchArchivedFaculties()
  })

  const days: { key: number; label: string }[] = [
    { key: 1, label: $t('mon') },
    { key: 2, label: $t('tue') },
    { key: 3, label: $t('wed') },
    { key: 4, label: $t('thu') },
    { key: 5, label: $t('fri') },
    { key: 6, label: $t('sat') },
    { key: 7, label: $t('sun') },
  ]

  $: lessons = $lessonsStore.filter(l => l.semesterId === semesterId)
  $: boundTeachers = $semesterTeachersStore.filter(t => t.isBound)
  $: dayToSlots = new Map<number, string[]>(
    days.map(d => {
      const union = Array.from(new Set(
        lessons
          .filter(l => l.dayOfWeek === d.key)
          .map(l => `${l.startTime}-${l.endTime}`)
      ))
      union.sort(compareSlots)
      return [d.key, union]
    })
  )

  let containerWidth = 0
  let dayHeaderEl: HTMLTableCellElement
  let timeHeaderEl: HTMLTableCellElement
  let facultyHeaderEl: HTMLTableCellElement
  let teacherCountHeaderEl: HTMLTableCellElement
  let deleteHeaderEl: HTMLTableCellElement
  const MIN_TEACHER_COL_PX = 96
  $: fixedColsTotalPx = (dayHeaderEl?.offsetWidth || 0) + (timeHeaderEl?.offsetWidth || 0) + (facultyHeaderEl?.offsetWidth || 0) + (teacherCountHeaderEl?.offsetWidth || 0) + (deleteHeaderEl?.offsetWidth || 0)
  $: teacherColsCount = boundTeachers.length
  $: teacherColWidthPx = Math.max(
    MIN_TEACHER_COL_PX,
    Math.floor(((containerWidth || 0) - (fixedColsTotalPx || 0)) / Math.max(1, teacherColsCount || 1))
  )

  function cellLessons(day: number, slot: string): Lesson[] {
    const [start, end] = slot.split('-')
    return lessons.filter(l => l.dayOfWeek === day && l.startTime === start && l.endTime === end)
  }

  function teacherCountForSlot(day: number, slot: string): { current: number; max: number | undefined } {
    const slotLessons = cellLessons(day, slot)
    if (slotLessons.length === 0) return { current: 0, max: undefined }
    // Берем максимальное значение teacher_count из всех пар в слоте
    const teacherCounts = slotLessons
      .map(l => l.teacherCount)
      .filter((c): c is number => c !== undefined)
    const maxCount = teacherCounts.length > 0 ? Math.max(...teacherCounts) : undefined
    // Подсчитываем уникальных преподавателей во всех парах слота
    const allTeacherIds = new Set(slotLessons.flatMap(l => l.teacherIds || []))
    return { current: allTeacherIds.size, max: maxCount }
  }

  function dirName(id: number): string {
    return $directionsStore.find(d => d.id === id)?.name || ''
  }

  function teachersNames(ids: number[]): string {
    return ids
      .map(id => $teachersStore.find(t => t.id === id))
      .filter(Boolean)
      .map(t => formatTeacherName(t!))
      .join(', ')
  }

  function facultyById(id: number): Faculty | undefined {
    return $facultiesStore.find(f => f.id === id) || archivedFaculties.find(f => f.id === id)
  }

  function facultyName(id: number): string {
    return facultyById(id)?.name || ''
  }

  function facultiesNames(ids: number[]): string {
    return ids.map(id => facultyName(id)).filter(Boolean).join(', ')
  }

  async function exportXLSX() {
    logUiEvent('click:export_xlsx', { screen: 'schedule', semesterId })
    if (!semesterId) return
    try {
      const resp = await App.ExportScheduleXLSX(semesterId)
      if (resp?.error) {
        alert(String(resp.error))
        logUiEvent('export_xlsx_error', { semesterId, error: resp.error })
        return
      }
      if (resp?.data) {
        logUiEvent('export_xlsx_success', { semesterId, path: resp.data })
      } else {
        logUiEvent('export_xlsx_cancelled', { semesterId })
      }
    } catch (e) {
      alert(String(e))
      logUiEvent('export_xlsx_exception', { semesterId, error: String(e) })
    }
  }

  let manageOpen = false
  let teacherSearch = ''
  let notificationMessage = ''
  let notificationTimeout: ReturnType<typeof setTimeout> | null = null

  function showNotification(message: string) {
    notificationMessage = message
    if (notificationTimeout) clearTimeout(notificationTimeout)
    notificationTimeout = setTimeout(() => {
      notificationMessage = ''
      notificationTimeout = null
    }, 3000)
  }

  onDestroy(() => {
    if (notificationTimeout) clearTimeout(notificationTimeout)
  })
  function openManage() { manageOpen = true; logUiEvent('click:open_manage_teachers', { semesterId }); loadSemesterTeachers(semesterId) }
  function closeManage() { manageOpen = false; logUiEvent('click:close_manage_teachers', { semesterId }) }
  $: filteredSemesterTeachersList = (() => {
    const q = teacherSearch.trim().toLowerCase()
    const list = $semesterTeachersStore
    if (!q) return list
    return list.filter(t => `${t.lastName} ${t.firstName} ${t.middleName ?? ''}`.toLowerCase().includes(q))
  })()
  async function toggleBind(st: SemesterTeacher) {
    logUiEvent('click:toggle_bind_teacher', { semesterId, teacherId: st.id, isBound: st.isBound })
    if (st.isBound) {
      await unbindTeacherFromSemester(semesterId, st.id)
    } else {
      const ok = await bindTeacherToSemester(semesterId, st.id)
      if (!ok) {
        alert($t('guest_only_one_semester'))
      }
    }
  }

  type Point = { x: number; y: number }
  let addSlotPopover: Point | null = null
  let addSlotDay: number = 1
  let addSlotStart = '09:40'
  let addSlotEnd = '11:15'
  let addSlotTeacherCount: number | undefined = undefined
  function openAddSlot(e: MouseEvent) {
    logUiEvent('click:open_add_slot', { semesterId })
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const width = 360
    const baseLeft = rect.left + (rect.width / 2) - (width / 2)
    const minLeft = 8
    const maxLeft = window.innerWidth - width - 8
    const left = Math.max(minLeft, Math.min(maxLeft, baseLeft))
    const top = rect.bottom + 8
    addSlotPopover = { x: left, y: top }
  }
  function closeAddSlot() { addSlotPopover = null; logUiEvent('click:close_add_slot', { semesterId }) }
  async function createTimeSlot() {
    if (!semesterId) return
    if (!addSlotTeacherCount || addSlotTeacherCount < 1) {
      alert($t('teacher_count_required'))
      return
    }
    const slot = normalizeSlot(`${addSlotStart}-${addSlotEnd}`)
    const current = dayToSlots.get(addSlotDay) || []
    if (current.includes(slot)) {
      alert($t('slot_exists'))
      return
    }
    let directionId = get(directionsStore)[0]?.id
    if (!directionId) {
      await refreshDirections()
      directionId = get(directionsStore)[0]?.id
    }
    if (!directionId) {
      alert($t('select_direction'))
      return
    }
    const { start, end } = parseSlot(slot)
    await addLesson({
      semesterId,
      dayOfWeek: addSlotDay as any,
      startTime: start,
      endTime: end,
      directionId,
      teacherIds: [],
      facultyIds: [],
      teacherCount: addSlotTeacherCount
    })
    logUiEvent('click:create_time_slot', { semesterId, day: addSlotDay, start: addSlotStart, end: addSlotEnd })
    addSlotPopover = null
  }

  function teacherLesson(day: number, slot: string, teacherId: number): Lesson | undefined {
    return cellLessons(day, slot).find(l => l.teacherIds.includes(teacherId))
  }

  async function toggleTeacherOnCell(day: number, slot: string, teacherId: number) {
    logUiEvent('click:toggle_teacher_on_cell', { semesterId, day, slot, teacherId })
    console.log(`toggleTeacherOnCell: day=${day}, slot=${slot}, teacherId=${teacherId}`)
    const existing = teacherLesson(day, slot, teacherId)
    const [start, end] = slot.split('-')
    console.log(`toggleTeacherOnCell: start=${start}, end=${end}, existing=${!!existing}`)

    if (existing) {
      const isLastTeacherInLesson = (existing.teacherIds?.length ?? 0) <= 1
      const slotLessonsAll = cellLessons(day, slot)
      const isOnlyLessonInSlot = slotLessonsAll.length <= 1

      if (!isLastTeacherInLesson) {
        const nextTeacherIds = (existing.teacherIds || []).filter(id => id !== teacherId)
        console.log('toggleTeacherOnCell: removing teacher from multi-teacher lesson', existing.id, nextTeacherIds)
        // Сохраняем teacher_count из существующей пары, если он был задан
        await updateLesson(existing.id, { teacherIds: nextTeacherIds, teacherCount: existing.teacherCount })
        return
      }

      if (isOnlyLessonInSlot) {
        console.log('toggleTeacherOnCell: converting last lesson in slot to placeholder', existing.id)
        // Сохраняем teacher_count при удалении всех преподавателей, чтобы можно было добавить их обратно
        await updateLesson(existing.id, { teacherIds: [], teacherCount: existing.teacherCount })
        return
      }

      console.log('toggleTeacherOnCell: deleting lesson (other lessons remain in slot)')
      await deleteLesson(existing.id)
      return
    }

    const slotLessons = cellLessons(day, slot)
    const blank = slotLessons.find(l => (l.teacherIds ?? []).length === 0)

    // Проверяем лимит преподавателей перед добавлением
    const countInfo = teacherCountForSlot(day, slot)
    // Если добавляем нового преподавателя, проверяем что текущее количество + 1 не превышает лимит
    const wouldExceedLimit = countInfo.max !== undefined && (countInfo.current + 1) > countInfo.max
    if (wouldExceedLimit) {
      // Показываем уведомление справа снизу
      showNotification($t('teacher_count_limit_reached').replace('{count}', String(countInfo.max)))
      return
    }

    const teacherRec = get(semesterTeachersStore).find(t => t.id === teacherId)

    let directionId = teacherRec?.directionId
    const dirList = get(directionsStore)
    const hasValidTeacherDirection = directionId && dirList.some(d => d.id === directionId)
    if (!hasValidTeacherDirection) {
      await refreshDirections()
    }
    const dirList2 = get(directionsStore)
    if (!hasValidTeacherDirection) {
      directionId = dirList2[0]?.id
    }
    if (!directionId) {
      alert($t('select_direction'))
      return
    }

    const facultyIds = facultiesForRow(day, slot)

    console.log(`toggleTeacherOnCell: teacherRec=${JSON.stringify(teacherRec)}, directionId=${directionId}, facultyIds=${facultyIds}`)

    // Используем teacher_count из blank пары, если она есть, иначе берем из других пар в слоте
    let teacherCount = blank?.teacherCount
    if (!teacherCount) {
      const slotLessons = cellLessons(day, slot)
      const teacherCounts = slotLessons
        .map(l => l.teacherCount)
        .filter((c): c is number => c !== undefined)
      if (teacherCounts.length > 0) {
        teacherCount = Math.max(...teacherCounts)
      } else {
        alert($t('teacher_count_required'))
        return
      }
    }

    const lessonData = {
      semesterId,
      dayOfWeek: day as any,
      startTime: start,
      endTime: end,
      directionId,
      teacherIds: [teacherId],
      facultyIds: facultyIds,
      teacherCount: teacherCount
    }

    console.log('toggleTeacherOnCell: calling addLesson with data:', lessonData)
    if (blank) {
      console.log('toggleTeacherOnCell: updating placeholder lesson', blank.id)
      const updated = await updateLesson(blank.id, {
        directionId,
        teacherIds: [teacherId],
        facultyIds: facultyIds,
        teacherCount: blank.teacherCount
      })
      if (!updated) {
        // Проверяем, не была ли это ошибка превышения лимита
        const countInfoAfter = teacherCountForSlot(day, slot)
        if (countInfoAfter.max !== undefined && countInfoAfter.current >= countInfoAfter.max) {
          showNotification($t('teacher_count_limit_reached').replace('{count}', String(countInfoAfter.max)))
        }
      }
      return
    }

    const result = await addLesson(lessonData)
    console.log('toggleTeacherOnCell: addLesson result:', result)
    if (!result) {
      // Проверяем, не была ли это ошибка превышения лимита
      const countInfoAfter = teacherCountForSlot(day, slot)
      if (countInfoAfter.max !== undefined && countInfoAfter.current >= countInfoAfter.max) {
        showNotification($t('teacher_count_limit_reached').replace('{count}', String(countInfoAfter.max)))
      }
    }
  }

  function rowKey(day: number, slot: string): string { return `${semesterId}-${day}-${slot}` }
  function facultiesForRow(day: number, slot: string): number[] {
    const union = Array.from(new Set(
      cellLessons(day, slot).flatMap(l => l.facultyIds)
    ))
    const valid = union.filter(id => $facultiesStore.some(f => f.id === id))
    return valid
  }
  async function toggleRowFaculty(day: number, slot: string, facultyId: number) {
    logUiEvent('click:toggle_row_faculty', { semesterId, day, slot, facultyId })
    const current = facultiesForRow(day, slot)
    const exists = current.includes(facultyId)
    const next = exists ? current.filter(id => id !== facultyId) : [...current, facultyId]
    const [start, end] = slot.split('-')
    const affected = $lessonsStore.filter(l => l.semesterId === semesterId && l.dayOfWeek === day && l.startTime === start && l.endTime === end)
    if (affected.length === 0) return
    await Promise.all(affected.map(l => updateLesson(l.id, { facultyIds: next })))
  }
  let facultyPopover: { day: number; slot: string; pos: Point } | null = null
  function openFacultyPopover(e: MouseEvent, day: number, slot: string) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const width = 360
    const estimatedHeight = 320
    const baseLeft = rect.left + (rect.width / 2) - (width / 2)
    const minLeft = 8
    const maxLeft = window.innerWidth - width - 8
    const left = Math.max(minLeft, Math.min(maxLeft, baseLeft))
    const spaceBelow = window.innerHeight - rect.bottom
    const openBelow = spaceBelow >= estimatedHeight + 8
    const top = openBelow ? rect.bottom + 8 : Math.max(8, rect.top - estimatedHeight - 8)
    facultyPopover = { day, slot, pos: { x: left, y: top } }
    logUiEvent('click:open_faculty_popover', { semesterId, day, slot, openBelow })
  }
  function closeFacultyPopover() { facultyPopover = null; logUiEvent('click:close_faculty_popover', { semesterId }) }

  function rateInfo(teacherId: number) {
    const RATE_FULL = 12
    const assigned = $lessonsStore.filter(l => l.semesterId === semesterId && l.teacherIds.includes(teacherId)).length
    const teacher = $semesterTeachersStore.find(t => t.id === teacherId)
    if (!teacher) return null
    const target = Math.round(teacher.rate * RATE_FULL)
    const remaining = target - assigned
    return { rate: teacher.rate, assigned, target, remaining, needed: Math.abs(remaining) }
  }

  function cellButtonClasses(assigned: boolean): string {
    const base = 'w-full h-8 rounded-md text-center text-base font-bold transition cursor-pointer select-none flex items-center justify-center ring-1 ring-black/5 dark:ring-white/10 '
    return assigned
      ? base + 'bg-indigo-100 hover:bg-indigo-200 text-indigo-900 dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:text-slate-100'
      : base + 'bg-slate-100 hover:bg-slate-200 text-transparent dark:bg-slate-900 dark:hover:bg-slate-800'
  }

  let lessonsVersion = 0
  const unsubscribeLessons = lessonsStore.subscribe(() => { lessonsVersion += 1 })
  onDestroy(unsubscribeLessons)

  function parseSlot(slot: string): { start: string; end: string } {
    const raw = String(slot ?? '')
    const [startRaw, endRaw] = raw.split('-')
    const norm = (v?: string) => {
      const s = String(v ?? '')
      const parts = s.split(':')
      if (parts.length >= 2) return `${(parts[0] ?? '').padStart(2, '0')}:${(parts[1] ?? '').padStart(2, '0')}`
      return s
    }
    return { start: norm(startRaw), end: norm(endRaw) }
  }

  function normalizeSlot(slot: string): string {
    const { start, end } = parseSlot(slot)
    return `${start}-${end}`
  }

  function compareSlots(a: string, b: string): number {
    const toMinutes = (t?: string) => {
      const s = String(t ?? '')
      const [h, m] = s.split(':')
      const hn = Number(h)
      const mn = Number(m)
      return (Number.isFinite(hn) ? hn : 0) * 60 + (Number.isFinite(mn) ? mn : 0)
    }
    const pa = parseSlot(a)
    const pb = parseSlot(b)
    return (toMinutes(pa.start) - toMinutes(pb.start)) || (toMinutes(pa.end) - toMinutes(pb.end))
  }
  async function deleteTimeSlotRow(day: number, slot: string) {
    const { start, end } = parseSlot(slot)
    const affected = $lessonsStore.filter(l => l.semesterId === semesterId && l.dayOfWeek === (day as any) && l.startTime === start && l.endTime === end)
    await Promise.all(affected.map(l => updateLesson(l.id, { facultyIds: [] })))
    await Promise.all(affected.map(l => deleteLesson(l.id)))
    logUiEvent('click:delete_time_slot_row', { semesterId, day, slot, affected: affected.map(l => l.id) })
  }
</script>

<div class="w-full h-screen flex flex-col px-4 py-4">
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-3">
      <button class="text-slate-500 hover:text-slate-700 dark:text-slate-300 dark:hover:text-white text-sm" on:click={back}>← {$t('back')}</button>
      {#if semester}
        <h2 class="text-2xl font-semibold">{semester.name}</h2>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      <button class="rounded-md bg-indigo-600 hover:bg-indigo-500 text-white px-3 py-2 text-xs" on:click={(e) => openAddSlot(e)} data-log-action="add_slot">{$t('add_slot')}</button>
      <button class="rounded-md bg-purple-600 hover:bg-purple-500 text-white px-3 py-2 text-xs" on:click={openManage} data-log-action="assign_teachers">{$t('assign_teachers')}</button>
      <button class="rounded-md bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white px-3 py-2 text-xs" on:click={exportXLSX} data-log-action="export_xlsx">{$t('export_xlsx')}</button>
    </div>
  </div>

  {#if addSlotPopover}
    <div class="fixed z-30 w-[360px] rounded-lg bg-white dark:bg-slate-900 ring-1 ring-black/10 dark:ring-white/10 p-4 shadow-xl" style={`left:${addSlotPopover.x}px; top:${addSlotPopover.y}px`} role="dialog" aria-modal="true">
      <div class="text-sm text-slate-700 dark:text-slate-300 mb-2">{$t('add_slot')}</div>
      <div class="grid gap-3 text-sm">
        <div class="flex items-center gap-2">
          <div class="text-slate-500 dark:text-slate-400 w-24">{$t('select_day')}</div>
          <div class="flex flex-wrap items-center gap-1 min-w-0">
            {#each days as d}
              <button class="px-1.5 py-1 text-xs rounded-md ring-1 ring-black/10 dark:ring-white/10 transition {addSlotDay === d.key ? 'bg-indigo-600 text-white' : 'bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700'}" on:click={() => { addSlotDay = d.key; logUiEvent('click:choose_day_for_slot', { semesterId, day: d.key }) }}>{d.label}</button>
            {/each}
          </div>
        </div>
        <div class="grid grid-cols-[max-content_1fr_max-content] grid-rows-2 gap-x-4 items-center min-w-0">
          <div class="text-slate-500 dark:text-slate-400 text-xs col-start-1 row-start-1">{$t('start_time')}</div>
          <div class="text-slate-500 dark:text-slate-400 text-xs col-start-3 row-start-1">{$t('end_time')}</div>
          <input class="w-24 rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10 col-start-1 row-start-2" bind:value={addSlotStart} on:change={() => logUiEvent('change:add_slot_start', { semesterId, value: addSlotStart })} />
          <div class="text-slate-400 text-center col-start-2 row-start-2 justify-self-center">—</div>
          <input class="w-24 rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10 col-start-3 row-start-2" bind:value={addSlotEnd} on:change={() => logUiEvent('change:add_slot_end', { semesterId, value: addSlotEnd })} />
        </div>
        <div class="flex items-center justify-between gap-2">
          <div class="text-slate-500 dark:text-slate-400 text-xs">{$t('teacher_count')}</div>
          <input type="number" min="1" class="w-24 rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10" bind:value={addSlotTeacherCount} on:change={() => logUiEvent('change:add_slot_teacher_count', { semesterId, value: addSlotTeacherCount })} placeholder={$t('empty')} />
        </div>
        <div class="flex items-center justify-end gap-2 mt-1">
          <button class="rounded-md bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600 px-3 py-1.5 text-xs" on:click={closeAddSlot}>{$t('cancel')}</button>
          <button class="rounded-md bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 text-white px-3 py-1.5 text-xs" on:click={createTimeSlot}>{$t('create')}</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="flex-1 overflow-auto custom-scroll" bind:clientWidth={containerWidth} style={`--teacher-col-width:${teacherColWidthPx}px`}>
    <table class="min-w-full text-sm border-separate w-auto" style="border-spacing: 0; table-layout: fixed;">
      <colgroup>
        <col style="width: 3rem" />
        <col style="width: 6rem" />
        <col style="width: 1%" />
        {#each boundTeachers as _}
          <col style="width: var(--teacher-col-width)" />
        {/each}
        <col style="width: 4rem" />
        <col style="width: 3.5rem" />
      </colgroup>
      <thead>
        <tr>
          <th class="sticky left-0 z-20 bg-white dark:bg-slate-900 text-center p-2 border-b border-slate-200 dark:border-slate-700 w-12" bind:this={dayHeaderEl}>{$t('day')}</th>
          <th class="sticky left-0 z-20 bg-white dark:bg-slate-900 text-center p-2 border-b border-slate-200 dark:border-slate-700 w-24" bind:this={timeHeaderEl}>{$t('time')}</th>
          <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-left whitespace-nowrap" bind:this={facultyHeaderEl}>{$t('faculty')}</th>
          {#each $semesterTeachersStore.filter(t => t.isBound) as teacher}
            <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-left align-top overflow-hidden" style="width: var(--teacher-col-width); min-width: var(--teacher-col-width); max-width: var(--teacher-col-width);">
              <div class="text-xs text-slate-500 dark:text-slate-400 break-words whitespace-normal min-w-0">{($directionsStore.find(d => d.id === teacher.directionId)?.name) || ''}</div>
              <div class="flex items-start gap-1">
                <div class="flex-1 min-w-0">
                  <div class="font-medium leading-tight text-sm break-words whitespace-normal" title={formatTeacherName(teacher)}>{formatTeacherName(teacher)}</div>
                  {#if rateInfo(teacher.id)}
                    {#key lessonsVersion + '-' + teacher.id}
                      <div class="text-[10px] text-slate-500 dark:text-slate-400 mt-1 truncate whitespace-nowrap">
                        {rateInfo(teacher.id).assigned}/{rateInfo(teacher.id).target}
                      </div>
                    {/key}
                  {/if}
                </div>
                {#if rateInfo(teacher.id)}
                  {#key lessonsVersion + '-' + teacher.id}
                    <div class="w-1.5 h-12 rounded bg-slate-300 dark:bg-slate-700 overflow-hidden flex-shrink-0 flex flex-col justify-end">
                      <div class="w-full transition-all duration-200"
                        class:bg-emerald-400={rateInfo(teacher.id).remaining === 0}
                        class:bg-amber-400={rateInfo(teacher.id).remaining < 0}
                        class:bg-indigo-500={rateInfo(teacher.id).remaining > 0}
                        style={`height: ${Math.min(100, Math.round((rateInfo(teacher.id).assigned / Math.max(1, rateInfo(teacher.id).target)) * 100))}%`}
                      />
                    </div>
                  {/key}
                {/if}
              </div>
            </th>
          {/each}
          <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-center whitespace-normal" bind:this={teacherCountHeaderEl}>
            <div class="text-xs leading-tight">{$t('teacher_count')}</div>
          </th>
          <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-right w-14" bind:this={deleteHeaderEl}></th>
        </tr>
      </thead>
      <tbody>
        {#each days as d}
          {#each (dayToSlots.get(d.key) || []) as slot, i}
            <tr>
              {#if i === 0}
                <td class="sticky left-0 z-10 bg-white dark:bg-slate-900 p-2 border-b border-slate-200 dark:border-slate-800 align-top text-center w-12" rowspan={(dayToSlots.get(d.key) || []).length}>{d.label}</td>
              {/if}
              <td class="sticky left-0 z-10 bg-white dark:bg-slate-900 p-2 border-b border-slate-200 dark:border-slate-800 font-medium text-center w-24">{slot}</td>
              <td class="p-2 border-b border-slate-200 dark:border-slate-800 whitespace-nowrap">
                {#key lessonsVersion + '-' + rowKey(d.key, slot)}
                  <div class="inline-flex flex-nowrap gap-1">
                    {#each facultiesForRow(d.key, slot) as fid}
                      {#if facultyById(fid)}
                        <span class="px-2 py-1 text-[10px] rounded bg-slate-200 dark:bg-slate-700 truncate max-w-[80px]" title={facultyName(fid)}>{facultyName(fid)}</span>
                      {/if}
                    {/each}
                    <button class="px-2 py-1 text-xs rounded bg-slate-100 hover:bg-slate-200 ring-1 ring-black/10 dark:bg-slate-800 dark:hover:bg-slate-700 dark:ring-white/10" on:click={(e) => openFacultyPopover(e, d.key, slot)} data-log-action="open_faculty_popover">+</button>
                  </div>
                {/key}
              </td>
              {#each $semesterTeachersStore.filter(t => t.isBound) as teacher}
                <td class="p-2 border-b border-slate-200 dark:border-slate-800 align-middle overflow-hidden" style="width: var(--teacher-col-width); min-width: var(--teacher-col-width); max-width: var(--teacher-col-width);">
                  {#key lessonsVersion + '-' + cellLessons(d.key, slot).map(l => l.id).join('-')}
                    <button
                      class={cellButtonClasses(!!teacherLesson(d.key, slot, teacher.id))}
                      on:click={() => toggleTeacherOnCell(d.key, slot, teacher.id)}
                      title={teacherLesson(d.key, slot, teacher.id) ? 'Снять назначение' : 'Назначить'}
                      aria-label={teacherLesson(d.key, slot, teacher.id) ? 'Снять назначение' : 'Назначить'}
                      data-log-action={teacherLesson(d.key, slot, teacher.id) ? 'unassign_teacher' : 'assign_teacher'}
                      data-teacher-id={teacher.id}
                      data-day={d.key}
                      data-slot={slot}
                    >
                      +
                    </button>
                  {/key}
                </td>
              {/each}
              <td class="p-2 border-b border-slate-200 dark:border-slate-800 text-center align-middle">
                {#key lessonsVersion + '-' + rowKey(d.key, slot)}
                  <div class="text-xs text-slate-600 dark:text-slate-400">
                    {(() => {
                      const countInfo = teacherCountForSlot(d.key, slot)
                      return countInfo.max !== undefined ? `${countInfo.current}/${countInfo.max}` : ''
                    })()}
                  </div>
                {/key}
              </td>
              <td class="p-2 border-b border-slate-200 dark:border-slate-800 text-right align-middle">
                <button class="px-2 py-1 text-xs rounded bg-rose-600/90 hover:bg-rose-600 text-white" title={$t('delete')} aria-label={$t('delete')} on:click={() => deleteTimeSlotRow(d.key, slot)} data-log-action="delete_time_slot">×</button>
              </td>
            </tr>
          {/each}
        {/each}
      </tbody>
    </table>
  </div>
  {#if manageOpen}
    <div class="fixed inset-0 z-30 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/40" role="button" tabindex="0" aria-label={$t('close')} on:click={closeManage} on:keydown={(e) => { if (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') { e.preventDefault(); closeManage(); } }} />
      <div class="relative w-[720px] max-w-[92vw] max-h-[80vh] overflow-auto rounded-lg bg-white dark:bg-slate-900 ring-1 ring-black/10 dark:ring-white/10 p-4 shadow-xl">
        <div class="flex items-center justify-between mb-3">
          <div class="text-sm font-medium">{$t('assign_teachers')}</div>
          <button class="px-2 py-1 text-xs rounded bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600" on:click={closeManage} data-log-action="close_manage_teachers">{$t('close')}</button>
        </div>
        <div class="mb-2">
          <input class="w-full rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10 text-sm" placeholder={$t('search_placeholder')} bind:value={teacherSearch} on:change={() => logUiEvent('change:teacher_search', { q: teacherSearch })} />
        </div>
        <div class="grid grid-cols-1 gap-1">
          {#each filteredSemesterTeachersList as st}
            <div class="flex items-center justify-between px-2 py-2 rounded ring-1 ring-black/10 dark:ring-white/10">
              <div class="min-w-0">
                <div class="text-sm font-medium truncate" title={`${st.lastName} ${st.firstName} ${st.middleName ?? ''}`}>{st.lastName} {st.firstName} {st.middleName}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400 flex items-center gap-2">
                  <span>{st.directionName || ''}</span>
                  {#if st.isGuest}<span class="px-1 rounded bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">Гость</span>{/if}
                  {#if st.isArchived}<span class="px-1 rounded bg-slate-200 text-slate-800 dark:bg-slate-800 dark:text-slate-200">Архив</span>{/if}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-slate-500 dark:text-slate-400">{st.isBound ? $t('assigned') : $t('empty')}</span>
                <button class="px-2 py-1 text-xs rounded {st.isBound ? 'bg-rose-600/90 hover:bg-rose-600 text-white' : 'bg-indigo-600 hover:bg-indigo-500 text-white'}" on:click={() => toggleBind(st)} data-log-action={st.isBound ? 'unbind_teacher' : 'bind_teacher'} data-teacher-id={st.id}>
                  {st.isBound ? $t('delete') : $t('add_teacher')}
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
  {#if facultyPopover}
    <div class="fixed z-30 w-[360px] max-h-[80vh] overflow-auto rounded-lg bg-white dark:bg-slate-900 ring-1 ring-black/10 dark:ring-white/10 p-3 shadow-xl" style={`left:${facultyPopover.pos.x}px; top:${facultyPopover.pos.y}px`} role="dialog" aria-modal="true">
      <div class="text-xs text-slate-600 dark:text-slate-400 mb-2">{$t('add_faculty')}</div>
      {#key lessonsVersion + '-' + rowKey(facultyPopover.day, facultyPopover.slot)}
        <div class="max-h-64 overflow-auto grid gap-1 mb-3">
          {#each $facultiesStore as f}
            <label class="flex items-center gap-2 text-sm">
              <input type="checkbox" class="accent-indigo-600" checked={facultiesForRow(facultyPopover.day, facultyPopover.slot).includes(f.id)} on:change={() => toggleRowFaculty(facultyPopover.day, facultyPopover.slot, f.id)} />
              <span>{f.name}</span>
            </label>
          {/each}
        </div>
      {/key}
      <div class="flex items-center justify-end gap-2">
        <button class="rounded-md bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600 px-3 py-1.5 text-xs" on:click={closeFacultyPopover}>{$t('close')}</button>
      </div>
    </div>
  {/if}
  {#if notificationMessage}
    <div class="fixed bottom-4 right-4 z-50 rounded-lg bg-rose-600 text-white px-4 py-3 shadow-lg ring-1 ring-black/10 dark:ring-white/10 animate-fade-in">
      <div class="text-sm font-medium">{notificationMessage}</div>
    </div>
  {/if}
  <style>
    :global(.custom-scroll) {
      scrollbar-color: rgba(99,102,241,0.5) rgba(255,255,255,0.06);
      scrollbar-width: thin;
    }
    :global(.custom-scroll::-webkit-scrollbar) {
      height: 8px;
      width: 8px;
    }
    :global(.custom-scroll::-webkit-scrollbar-track) {
      background: linear-gradient(90deg, rgba(255,255,255,0.08), rgba(255,255,255,0.12));
      border-radius: 999px;
    }
    :global(.custom-scroll::-webkit-scrollbar-thumb) {
      background: linear-gradient(180deg, rgba(165,180,252,0.9), rgba(129,140,248,0.9));
      border-radius: 999px;
      border: 1px solid rgba(255,255,255,0.25);
    }
    :global(.custom-scroll::-webkit-scrollbar-thumb:hover) {
      background: linear-gradient(180deg, rgba(199,210,254,1), rgba(165,180,252,1));
    }
    @keyframes fade-in {
      from {
        opacity: 0;
        transform: translateY(10px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
    .animate-fade-in {
      animation: fade-in 0.2s ease-out;
    }
  </style>
</div>
