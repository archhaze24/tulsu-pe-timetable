<script lang="ts">
  import { t } from 'svelte-i18n'
  import { facultiesStore } from '../../stores/faculties'
  import { directionsStore } from '../../stores/directions'
  import { teachersStore, formatTeacherName } from '../../stores/teachers'
  import { semesterTeachersStore, loadSemesterTeachers, bindTeacherToSemester, unbindTeacherFromSemester, type SemesterTeacher } from '../../stores/semester-teachers'
  import { lessonsStore, semestersStore, addLesson, deleteLesson, updateLesson, loadLessonsBySemester, loadSemesters, type Lesson } from '../../stores/semesters'
  import { navigate, route } from '../../stores/router'
  import { onDestroy } from 'svelte'
  import { get } from 'svelte/store'

  const back = () => navigate('semesters')

  $: semesterId = Number($route.params?.id ?? 0) || $semestersStore[0]?.id || 0
  $: if (semesterId) { loadSemesterTeachers(semesterId) }
  $: if (semesterId) { loadLessonsBySemester(semesterId) }
  // Ensure semesters list is present if user navigates directly
  $: if (!$semestersStore || $semestersStore.length === 0) { loadSemesters() }
  $: semester = $semestersStore.find(s => s.id === semesterId)

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
  // Additional slots created before assigning any lesson
  let extraSlots: Set<string> = new Set()
  function slotKey(semId: number, day: number, slot: string): string { return `${semId}:${day}:${slot}` }
  $: dayToSlots = new Map<number, string[]>(
    days.map(d => {
      const fromLessons = Array.from(new Set(
        lessons
          .filter(l => l.dayOfWeek === d.key)
          .map(l => `${l.startTime}-${l.endTime}`)
      ))
      const fromExtras = Array.from(new Set(
        Array.from(extraSlots)
          .filter(s => s.startsWith(`${semesterId}:${d.key}:`))
          .map(s => s.split(':')[2])
      ))
      const union = Array.from(new Set([...fromLessons, ...fromExtras]))
      union.sort()
      return [d.key, union]
    })
  )

  function cellLessons(day: number, slot: string): Lesson[] {
    const [start, end] = slot.split('-')
    return lessons.filter(l => l.dayOfWeek === day && l.startTime === start && l.endTime === end)
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

  function facultiesNames(ids: number[]): string {
    return ids
      .map(id => $facultiesStore.find(f => f.id === id)?.name || '')
      .join(', ')
  }

  function exportXLSX() {
    alert('Экспорт XLSX будет реализован на бэкенде')
  }

  // Save schedule action (stub – integrate with backend via Wails later)
  function saveSchedule() {
    alert('Расписание сохранено')
  }

  // Manage teachers modal
  let manageOpen = false
  let teacherSearch = ''
  function openManage() { manageOpen = true; loadSemesterTeachers(semesterId) }
  function closeManage() { manageOpen = false }
  $: filteredSemesterTeachersList = (() => {
    const q = teacherSearch.trim().toLowerCase()
    const list = $semesterTeachersStore
    if (!q) return list
    return list.filter(t => `${t.lastName} ${t.firstName} ${t.middleName ?? ''}`.toLowerCase().includes(q))
  })()
  async function toggleBind(st: SemesterTeacher) {
    if (st.isBound) {
      await unbindTeacherFromSemester(semesterId, st.id)
    } else {
      const ok = await bindTeacherToSemester(semesterId, st.id)
      if (!ok) {
        alert($t('guest_only_one_semester'))
      }
    }
  }

  // Add time slot popover (anchored to header button)
  type Point = { x: number; y: number }
  let addSlotPopover: Point | null = null
  let addSlotDay: number = 1
  let addSlotStart = '09:40'
  let addSlotEnd = '11:15'
  function openAddSlot(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const width = 360
    const baseLeft = rect.left + (rect.width / 2) - (width / 2)
    const minLeft = 8
    const maxLeft = window.innerWidth - width - 8
    const left = Math.max(minLeft, Math.min(maxLeft, baseLeft))
    const top = rect.bottom + 8
    addSlotPopover = { x: left, y: top }
  }
  function closeAddSlot() { addSlotPopover = null }
  function createTimeSlot() {
    if (!semesterId) return
    const slot = `${addSlotStart}-${addSlotEnd}`
    const current = dayToSlots.get(addSlotDay) || []
    if (current.includes(slot)) {
      alert($t('slot_exists'))
      return
    }
    extraSlots.add(slotKey(semesterId, addSlotDay as any, slot))
    addSlotPopover = null
  }

  function teacherLesson(day: number, slot: string, teacherId: number): Lesson | undefined {
    return cellLessons(day, slot).find(l => l.teacherIds.includes(teacherId))
  }

  async function toggleTeacherOnCell(day: number, slot: string, teacherId: number) {
    const existing = teacherLesson(day, slot, teacherId)
    const [start, end] = slot.split('-')
    if (existing) {
      await deleteLesson(existing.id)
      return
    }
    const teacherRec = get(semesterTeachersStore).find(t => t.id === teacherId)
    const directionId = teacherRec?.directionId || (get(directionsStore)[0]?.id || 1)
    await addLesson({
      semesterId,
      dayOfWeek: day as any,
      startTime: start,
      endTime: end,
      directionId,
      teacherIds: [teacherId],
      facultyIds: facultiesForRow(day, slot),
      teacherCount: 1
    })
  }

  // Faculties per row (day+slot) mock management
  function rowKey(day: number, slot: string): string { return `${semesterId}-${day}-${slot}` }
  let rowFacultyMap: Map<string, number[]> = new Map()
  function facultiesForRow(day: number, slot: string): number[] {
    const key = rowKey(day, slot)
    if (rowFacultyMap.has(key)) return rowFacultyMap.get(key) as number[]
    const union = Array.from(new Set(
      cellLessons(day, slot).flatMap(l => l.facultyIds)
    ))
    // Filter out ids that don't exist in store to avoid empty chips
    const valid = union.filter(id => $facultiesStore.some(f => f.id === id))
    rowFacultyMap.set(key, valid)
    return valid
  }
  async function toggleRowFaculty(day: number, slot: string, facultyId: number) {
    const key = rowKey(day, slot)
    const current = facultiesForRow(day, slot)
    const exists = current.includes(facultyId)
    const next = exists ? current.filter(id => id !== facultyId) : [...current, facultyId]
    rowFacultyMap.set(key, next)
    rowFacultyVersion += 1
    // propagate mock change into lessons on that row
    const [start, end] = slot.split('-')
    const affected = $lessonsStore.filter(l => l.semesterId === semesterId && l.dayOfWeek === day && l.startTime === start && l.endTime === end)
    if (affected.length === 0) return
    await Promise.all(affected.map(l => updateLesson(l.id, { facultyIds: next })))
  }
  // Single floating faculty popover anchored to the "+" button
  let facultyPopover: { day: number; slot: string; pos: Point } | null = null
  function openFacultyPopover(e: MouseEvent, day: number, slot: string) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const width = 360
    const baseLeft = rect.left + (rect.width / 2) - (width / 2)
    const minLeft = 8
    const maxLeft = window.innerWidth - width - 8
    const left = Math.max(minLeft, Math.min(maxLeft, baseLeft))
    const top = rect.bottom + 8
    facultyPopover = { day, slot, pos: { x: left, y: top } }
  }
  function closeFacultyPopover() { facultyPopover = null }
  // Version to force re-render when faculties map changes
  let rowFacultyVersion = 0

  // Rate info per teacher (12 pairs/week = 1.0 rate)
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
    const base = 'w-full h-8 rounded-md text-center text-xs transition cursor-pointer select-none flex items-center justify-center ring-1 ring-black/5 dark:ring-white/10 '
    return assigned
      ? base + 'bg-indigo-100 hover:bg-indigo-200 text-indigo-900 dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:text-slate-100'
      : base + 'bg-slate-100 hover:bg-slate-200 text-transparent dark:bg-slate-900 dark:hover:bg-slate-800'
  }

  // Force re-render for lessons changes so "+" and rates update instantly
  let lessonsVersion = 0
  const unsubscribeLessons = lessonsStore.subscribe(() => { lessonsVersion += 1 })
  onDestroy(unsubscribeLessons)

  // Helpers for time slots
  function parseSlot(slot: string): { start: string; end: string } {
    const [start, end] = slot.split('-')
    const norm = (s: string) => {
      const parts = s.split(':')
      if (parts.length >= 2) return `${parts[0].padStart(2, '0')}:${parts[1].padStart(2, '0')}`
      return s
    }
    return { start: norm(start), end: norm(end) }
  }
  async function deleteTimeSlotRow(day: number, slot: string) {
    const { start, end } = parseSlot(slot)
    const affected = $lessonsStore.filter(l => l.semesterId === semesterId && l.dayOfWeek === (day as any) && l.startTime === start && l.endTime === end)
    await Promise.all(affected.map(l => deleteLesson(l.id)))
    // remove extra-only slot
    extraSlots.delete(slotKey(semesterId, day as any, slot))
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
      <button class="rounded-md bg-indigo-600 hover:bg-indigo-500 text-white px-3 py-2 text-xs" on:click={(e) => openAddSlot(e)}>{$t('add_slot')}</button>
      <button class="rounded-md bg-purple-600 hover:bg-purple-500 text-white px-3 py-2 text-xs" on:click={openManage}>{$t('assign_teachers')}</button>
      <button class="rounded-md bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 text-white px-3 py-2 text-xs" on:click={saveSchedule}>{$t('save_schedule')}</button>
      <button class="rounded-md bg-indigo-600 hover:bg-indigo-500 text-white px-3 py-2 text-xs" on:click={exportXLSX}>{$t('export_xlsx')}</button>
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
              <button class="px-1.5 py-1 text-xs rounded-md ring-1 ring-black/10 dark:ring-white/10 transition {addSlotDay === d.key ? 'bg-indigo-600 text-white' : 'bg-slate-100 hover:bg-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700'}" on:click={() => addSlotDay = d.key}>{d.label}</button>
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
        <div class="flex items-center justify-end gap-2 mt-1">
          <button class="rounded-md bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600 px-3 py-1.5 text-xs" on:click={closeAddSlot}>{$t('cancel')}</button>
          <button class="rounded-md bg-emerald-500 hover:bg-emerald-400 active:bg-emerald-600 text-white px-3 py-1.5 text-xs" on:click={createTimeSlot}>{$t('create')}</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="flex-1 overflow-auto custom-scroll">
    <table class="min-w-full text-sm border-separate w-full" style="border-spacing: 0; table-layout: fixed;">
      <thead>
        <tr>
          <th class="sticky left-0 z-20 bg-white dark:bg-slate-900 text-center p-2 border-b border-slate-200 dark:border-slate-700 w-12">{$t('day')}</th>
          <th class="sticky left-0 z-20 bg-white dark:bg-slate-900 text-center p-2 border-b border-slate-200 dark:border-slate-700 w-24">{$t('time')}</th>
          <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-left w-24">{$t('faculty')}</th>
          {#each $semesterTeachersStore.filter(t => t.isBound) as teacher}
            <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-left w-20">
              <div class="text-xs text-slate-500 dark:text-slate-400">{($directionsStore.find(d => d.id === teacher.directionId)?.name) || ''}</div>
              <div class="flex items-start gap-1">
                <div class="flex-1 min-w-0">
                  <div class="font-medium leading-tight text-sm break-words" title={formatTeacherName(teacher)}>{formatTeacherName(teacher)}</div>
                  {#if rateInfo(teacher.id)}
                    {#key lessonsVersion + '-' + teacher.id}
                      <div class="text-[10px] text-slate-500 dark:text-slate-400 mt-1">
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
          <th class="p-2 border-b border-slate-200 dark:border-slate-700 text-right w-14"></th>
        </tr>
      </thead>
      <tbody>
        {#each days as d}
          {#each (dayToSlots.get(d.key) || []) as slot, i}
            <tr class="align-top">
              {#if i === 0}
                <td class="sticky left-0 z-10 bg-white dark:bg-slate-900 p-2 border-b border-slate-200 dark:border-slate-800 align-top text-center w-12" rowspan={(dayToSlots.get(d.key) || []).length}>{d.label}</td>
              {/if}
              <td class="sticky left-0 z-10 bg-white dark:bg-slate-900 p-2 border-b border-slate-200 dark:border-slate-800 font-medium text-center w-24">{slot}</td>
              <td class="p-2 border-b border-slate-200 dark:border-slate-800 w-24">
                {#key rowFacultyVersion + '-' + rowKey(d.key, slot)}
                  <div class="flex flex-wrap gap-1">
                    {#each facultiesForRow(d.key, slot) as fid}
                      {#if $facultiesStore.find(f => f.id === fid)}
                        <span class="px-2 py-1 text-[10px] rounded bg-slate-200 dark:bg-slate-700 truncate max-w-[80px]" title={$facultiesStore.find(f => f.id === fid)?.name}>{$facultiesStore.find(f => f.id === fid)?.name}</span>
                      {/if}
                    {/each}
                    <button class="px-2 py-1 text-xs rounded bg-slate-100 hover:bg-slate-200 ring-1 ring-black/10 dark:bg-slate-800 dark:hover:bg-slate-700 dark:ring-white/10" on:click={(e) => openFacultyPopover(e, d.key, slot)}>+</button>
                  </div>
                {/key}
              </td>
              {#each $semesterTeachersStore.filter(t => t.isBound) as teacher}
                <td class="p-2 border-b border-slate-200 dark:border-slate-800 w-20">
                  {#key lessonsVersion + '-' + cellLessons(d.key, slot).map(l => l.id).join('-')}
                    <button
                      class={cellButtonClasses(!!teacherLesson(d.key, slot, teacher.id))}
                      on:click={() => toggleTeacherOnCell(d.key, slot, teacher.id)}
                      title={teacherLesson(d.key, slot, teacher.id) ? 'Снять назначение' : 'Назначить'}
                      aria-label={teacherLesson(d.key, slot, teacher.id) ? 'Снять назначение' : 'Назначить'}
                    >
                      +
                    </button>
                  {/key}
                </td>
              {/each}
              <td class="p-2 border-b border-slate-200 dark:border-slate-800 text-right align-middle">
                <button class="px-2 py-1 text-xs rounded bg-rose-600/90 hover:bg-rose-600 text-white" title={$t('delete')} aria-label={$t('delete')} on:click={() => deleteTimeSlotRow(d.key, slot)}>×</button>
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
          <button class="px-2 py-1 text-xs rounded bg-slate-200 hover:bg-slate-300 dark:bg-slate-700 dark:hover:bg-slate-600" on:click={closeManage}>{$t('close')}</button>
        </div>
        <div class="mb-2">
          <input class="w-full rounded bg-white dark:bg-slate-800 px-2 py-1 ring-1 ring-slate-300 dark:ring-white/10 text-sm" placeholder={$t('search_placeholder')} bind:value={teacherSearch} />
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
                <button class="px-2 py-1 text-xs rounded {st.isBound ? 'bg-rose-600/90 hover:bg-rose-600 text-white' : 'bg-indigo-600 hover:bg-indigo-500 text-white'}" on:click={() => toggleBind(st)}>
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
    <div class="fixed z-30 w-[360px] rounded-lg bg-white dark:bg-slate-900 ring-1 ring-black/10 dark:ring-white/10 p-3 shadow-xl" style={`left:${facultyPopover.pos.x}px; top:${facultyPopover.pos.y}px`} role="dialog" aria-modal="true">
      <div class="text-xs text-slate-600 dark:text-slate-400 mb-2">{$t('add_faculty')}</div>
      {#key rowFacultyVersion + '-' + rowKey(facultyPopover.day, facultyPopover.slot)}
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
  </style>
</div>
