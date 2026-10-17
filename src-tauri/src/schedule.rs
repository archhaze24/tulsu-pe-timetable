//! Правила расписания. Фронтенд только отображает готовую модель и вызывает операции.
//! Слот — строка расписания «ЧЧ:ММ-ЧЧ:ММ» в дне недели. Пары слота — занятия с этим днём и интервалом.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::Connection;
use serde::Serialize;
use ts_rs::TS;

use crate::error::{AppError, AppResult};
use crate::logger;
use crate::models::{Direction, Faculty, Lesson, LessonInput};
use crate::storage::repo;

/// Стандартные пары ТулГУ по 95 минут. Другое время допустимо только с признаком «нестандартное».
pub const STANDARD_PAIRS: [(&str, &str); 5] = [
    ("07:45", "09:20"),
    ("09:40", "11:15"),
    ("11:35", "13:10"),
    ("13:40", "15:15"),
    ("15:35", "17:10"),
];

/// Полная ставка — 12 занятий в неделю (правило предметной области).
const FULL_RATE_LESSONS: f64 = 12.0;

/// Строгий разбор «ЧЧ:ММ». Возвращает минуты от начала суток.
pub fn parse_time(value: &str) -> Option<u32> {
    let bytes = value.as_bytes();
    let well_formed = bytes.len() == 5
        && bytes[2] == b':'
        && bytes.iter().enumerate().all(|(i, b)| i == 2 || b.is_ascii_digit());
    if !well_formed {
        return None;
    }
    let hours: u32 = value[..2].parse().ok()?;
    let minutes: u32 = value[3..].parse().ok()?;
    (hours < 24 && minutes < 60).then_some(hours * 60 + minutes)
}

/// Приводит «9:40» к «09:40» для данных старых версий. Прочее возвращает без изменений.
pub fn normalize_time(value: &str) -> String {
    let trimmed = value.trim();
    match trimmed.split_once(':') {
        Some((hours, minutes))
            if (1..=2).contains(&hours.len())
                && minutes.len() == 2
                && hours.bytes().chain(minutes.bytes()).all(|b| b.is_ascii_digit()) =>
        {
            format!("{hours:0>2}:{minutes}")
        }
        _ => trimmed.to_owned(),
    }
}

/// Минуты для сортировки. Некорректное время уходит в конец списка.
pub fn time_minutes(value: &str) -> u32 {
    parse_time(value).unwrap_or(u32::MAX)
}

fn parse_slot(slot: &str) -> Option<(String, String)> {
    let (start, end) = slot.split_once('-')?;
    let (start_min, end_min) = (parse_time(start)?, parse_time(end)?);
    if start_min >= end_min {
        return None;
    }
    Some((start.to_owned(), end.to_owned()))
}

fn in_cell(lesson: &Lesson, day: i64, start: &str, end: &str) -> bool {
    lesson.day_of_week == day && lesson.start_time == start && lesson.end_time == end
}

fn overlaps(a_start: u32, a_end: u32, b_start: u32, b_end: u32) -> bool {
    a_start < b_end && b_start < a_end
}

fn check_day(day: i64) -> AppResult<()> {
    if (1..=7).contains(&day) {
        Ok(())
    } else {
        Err(AppError::key("errors.schedule.day_invalid"))
    }
}

/// Результат операции расписания. Ошибки бизнес-правил — коды, тексты берёт фронтенд из локалей.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ScheduleOutcome {
    Ok,
    TeacherCountExceeded { max: i64 },
    TeacherCountBelowAssigned { assigned: i64 },
    TeacherCountInvalid,
    SlotExists,
    SlotOverlaps,
    SlotNotFound,
    InvalidTime,
    StartNotBeforeEnd,
    NotStandardPair,
    SelectDirection,
    DirectionArchived,
    TeacherNotBound,
}

/// Итог снятия преподавателя: сколько назначений снято и сколько пар удалено целиком.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct UnbindResult {
    pub assignments: usize,
    pub deleted_pairs: usize,
}

/// Модель расписания для экрана: слоты по дням и ячейки, уже посчитанные на бэкенде.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct ScheduleView {
    pub days: Vec<DayView>,
    pub teachers: Vec<TeacherView>,
    /// Все факультеты, включая архивные: их названия нужны в ячейках слотов.
    pub faculties: Vec<Faculty>,
    pub directions: Vec<Direction>,
    pub standard_pairs: Vec<TimePair>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct TimePair {
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct DayView {
    pub day: i64,
    pub slots: Vec<SlotView>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct SlotView {
    pub slot: String,
    pub start_time: String,
    pub end_time: String,
    /// Факультеты строки: объединение факультетов всех пар слота.
    pub faculty_ids: Vec<i64>,
    /// Лимит по `teacher_count`; `None`, если пар с лимитом нет.
    pub teacher_max: Option<i64>,
    /// Число уникальных назначенных преподавателей слота.
    pub teacher_current: usize,
    /// Преподаватели, назначенные в слоте (по id).
    pub assigned_teacher_ids: Vec<i64>,
    /// Число пар в слоте (для подтверждения удаления).
    pub pair_count: usize,
    /// Число назначений «преподаватель в паре» (для подтверждения очистки).
    pub assignment_count: usize,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct TeacherView {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
    pub direction_id: i64,
    pub direction_name: String,
    pub direction_archived: bool,
    pub rate: f64,
    /// Занятий в семестре (назначено пар с этим преподавателем).
    pub assigned: usize,
    /// Целевое число занятий: `rate * 12`.
    pub target: usize,
}

/// Строит модель расписания семестра: слоты дней, ячейки и ставки привязанных преподавателей.
pub fn build_view(conn: &Connection, semester_id: i64) -> AppResult<ScheduleView> {
    let lessons = repo::lessons_for_semester(conn, semester_id)?;
    let teachers: Vec<TeacherView> = repo::teachers_for_semester(conn, semester_id)?
        .into_iter()
        .filter(|t| t.is_bound)
        .map(|t| {
            let assigned = lessons.iter().filter(|l| l.teacher_ids.contains(&t.id)).count();
            TeacherView {
                id: t.id,
                first_name: t.first_name,
                last_name: t.last_name,
                middle_name: t.middle_name,
                direction_id: t.direction_id,
                direction_name: t.direction_name,
                direction_archived: t.direction_archived,
                rate: t.rate,
                assigned,
                target: (t.rate * FULL_RATE_LESSONS).round() as usize,
            }
        })
        .collect();

    let days = (1..=7)
        .map(|day| DayView { day, slots: slots_for_day(&lessons, day) })
        .collect();

    Ok(ScheduleView {
        days,
        teachers,
        faculties: repo::all_faculties(conn)?,
        directions: repo::list_directions(conn, false)?,
        standard_pairs: STANDARD_PAIRS
            .iter()
            .map(|(start, end)| TimePair { start_time: start.to_string(), end_time: end.to_string() })
            .collect(),
    })
}

fn slots_for_day(lessons: &[Lesson], day: i64) -> Vec<SlotView> {
    let mut cells: BTreeMap<(u32, u32), (String, String)> = BTreeMap::new();
    for lesson in lessons.iter().filter(|l| l.day_of_week == day) {
        cells
            .entry((time_minutes(&lesson.start_time), time_minutes(&lesson.end_time)))
            .or_insert_with(|| (lesson.start_time.clone(), lesson.end_time.clone()));
    }

    cells
        .into_values()
        .map(|(start, end)| {
            let pairs: Vec<&Lesson> = lessons.iter().filter(|l| in_cell(l, day, &start, &end)).collect();
            let faculty_ids: BTreeSet<i64> = pairs.iter().flat_map(|l| l.faculty_ids.iter().copied()).collect();
            let assigned: BTreeSet<i64> = pairs.iter().flat_map(|l| l.teacher_ids.iter().copied()).collect();
            SlotView {
                slot: format!("{start}-{end}"),
                start_time: start.clone(),
                end_time: end.clone(),
                faculty_ids: faculty_ids.into_iter().collect(),
                teacher_max: pairs.iter().filter_map(|l| l.teacher_count).max(),
                teacher_current: assigned.len(),
                assigned_teacher_ids: assigned.into_iter().collect(),
                pair_count: pairs.len(),
                assignment_count: pairs.iter().map(|l| l.teacher_ids.len()).sum(),
            }
        })
        .collect()
}

/// Направление для новой пары — направление преподавателя.
fn lesson_direction(conn: &Connection, teacher_id: i64) -> AppResult<Result<i64, ScheduleOutcome>> {
    match repo::teacher_direction(conn, teacher_id)? {
        (0, _) | (_, None) => Ok(Err(ScheduleOutcome::SelectDirection)),
        (_, Some(true)) => Ok(Err(ScheduleOutcome::DirectionArchived)),
        (id, Some(false)) => Ok(Ok(id)),
    }
}

/// Изменение пары при снятии преподавателя. План считается на копии данных, применяется в транзакции.
#[derive(Debug)]
enum Change {
    Update { lesson_id: i64, input: LessonInput },
    Delete { lesson_id: i64 },
}

/// План снятия преподавателя из пар переданного набора.
///
/// Если в паре несколько преподавателей, снимаем только его. Если он единственный и в слоте
/// одна пара, оставляем пустую заглушку с тем же лимитом. Иначе удаляем пару целиком.
fn plan_removal(lessons: &[Lesson], teacher_id: i64) -> Vec<Change> {
    let mut model: Vec<Lesson> = lessons.to_vec();
    let mut changes = Vec::new();
    while let Some(idx) = model.iter().position(|l| l.teacher_ids.contains(&teacher_id)) {
        let (day, start, end) = (model[idx].day_of_week, model[idx].start_time.clone(), model[idx].end_time.clone());
        let cell_size = model.iter().filter(|l| in_cell(l, day, &start, &end)).count();
        if model[idx].teacher_ids.len() > 1 {
            model[idx].teacher_ids.retain(|id| *id != teacher_id);
        } else if cell_size <= 1 {
            model[idx].teacher_ids.clear();
        } else {
            let lesson_id = model[idx].id;
            model.remove(idx);
            changes.push(Change::Delete { lesson_id });
            continue;
        }
        changes.push(Change::Update { lesson_id: model[idx].id, input: input_from(&model[idx]) });
    }
    changes
}

fn apply_changes(conn: &Connection, changes: &[Change]) -> AppResult<UnbindResult> {
    let mut deleted_pairs = 0;
    for change in changes {
        match change {
            Change::Update { lesson_id, input } => repo::update_lesson(conn, *lesson_id, input)?,
            Change::Delete { lesson_id } => {
                repo::delete_lesson(conn, *lesson_id)?;
                deleted_pairs += 1;
            }
        }
    }
    Ok(UnbindResult { assignments: changes.len(), deleted_pairs })
}

/// Снимает преподавателя со всех назначений семестра. Вызывать внутри транзакции.
pub fn remove_assignments(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<UnbindResult> {
    let lessons = repo::lessons_for_semester(conn, semester_id)?;
    apply_changes(conn, &plan_removal(&lessons, teacher_id))
}

/// Что будет снято при отвязке, без изменений в базе.
pub fn preview_unbind(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<UnbindResult> {
    let lessons = repo::lessons_for_semester(conn, semester_id)?;
    let changes = plan_removal(&lessons, teacher_id);
    Ok(UnbindResult {
        assignments: changes.len(),
        deleted_pairs: changes.iter().filter(|c| matches!(c, Change::Delete { .. })).count(),
    })
}

/// Отвязывает преподавателя от семестра и снимает все его назначения в этом семестре.
pub fn unbind_teacher(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<UnbindResult> {
    let tx = repo::begin(conn)?;
    let removed = remove_assignments(&tx, semester_id, teacher_id)?;
    repo::delete_binding(&tx, semester_id, teacher_id)?;
    commit(tx)?;
    logger::info(
        "schedule",
        format!(
            "преподаватель id={teacher_id} отвязан от семестра id={semester_id}: назначений снято {}, пар удалено {}",
            removed.assignments, removed.deleted_pairs
        ),
    );
    Ok(removed)
}

/// Переключает преподавателя в ячейке слота.
///
/// Снятие: см. `plan_removal`. Назначение: переиспользуем пустую заглушку слота, иначе создаём новую пару.
pub fn toggle_teacher(conn: &Connection, semester_id: i64, day: i64, slot: &str, teacher_id: i64) -> AppResult<ScheduleOutcome> {
    let Some((start, end)) = parse_slot(slot) else { return Ok(ScheduleOutcome::InvalidTime) };
    check_day(day)?;

    let tx = repo::begin(conn)?;
    let lessons = repo::lessons_for_semester(&tx, semester_id)?;
    let cell: Vec<Lesson> = lessons.iter().filter(|l| in_cell(l, day, &start, &end)).cloned().collect();
    if cell.is_empty() {
        return Ok(ScheduleOutcome::SlotNotFound);
    }

    if cell.iter().any(|l| l.teacher_ids.contains(&teacher_id)) {
        let removed = apply_changes(&tx, &plan_removal(&cell, teacher_id))?;
        commit(tx)?;
        logger::info(
            "schedule",
            format!("снят преподаватель id={teacher_id} из слота {slot} (день {day}): назначений {}, пар удалено {}", removed.assignments, removed.deleted_pairs),
        );
        return Ok(ScheduleOutcome::Ok);
    }

    if !repo::is_bound(&tx, semester_id, teacher_id)? {
        return Ok(ScheduleOutcome::TeacherNotBound);
    }
    let direction_id = match lesson_direction(&tx, teacher_id)? {
        Ok(id) => id,
        Err(outcome) => return Ok(outcome),
    };

    let max = cell.iter().filter_map(|l| l.teacher_count).max();
    let assigned: BTreeSet<i64> = cell.iter().flat_map(|l| l.teacher_ids.iter().copied()).collect();
    if let Some(max) = max && assigned.len() as i64 + 1 > max {
        logger::warn("schedule", format!("лимит преподавателей слота {slot} (день {day}) исчерпан: max={max}"));
        return Ok(ScheduleOutcome::TeacherCountExceeded { max });
    }

    let faculty_ids: Vec<i64> = cell
        .iter()
        .flat_map(|l| l.faculty_ids.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let blank = cell.iter().find(|l| l.teacher_ids.is_empty());
    let Some(teacher_count) = blank.and_then(|l| l.teacher_count).or(max) else {
        return Ok(ScheduleOutcome::TeacherCountInvalid);
    };

    let input = LessonInput {
        semester_id,
        day_of_week: day,
        start_time: start.clone(),
        end_time: end.clone(),
        direction_id,
        teacher_count: Some(teacher_count),
        faculty_ids,
        teacher_ids: vec![teacher_id],
    };
    match blank {
        Some(lesson) => {
            repo::update_lesson(&tx, lesson.id, &input)?;
            logger::info("schedule", format!("назначен преподаватель id={teacher_id} в заглушку пары id={}", lesson.id));
        }
        None => {
            let id = repo::create_lesson(&tx, &input)?;
            logger::info("schedule", format!("создана пара id={id}: день {day}, {slot}, преподаватель id={teacher_id}"));
        }
    }
    commit(tx)?;
    Ok(ScheduleOutcome::Ok)
}

/// Заменяет факультеты строки во всех парах слота.
pub fn set_slot_faculties(conn: &Connection, semester_id: i64, day: i64, slot: &str, faculty_ids: &[i64]) -> AppResult<()> {
    let (start, end) = parse_slot(slot).ok_or_else(|| AppError::key("errors.schedule.slot_not_found"))?;
    check_day(day)?;
    let wanted: Vec<i64> = faculty_ids.iter().copied().collect::<BTreeSet<_>>().into_iter().collect();

    let tx = repo::begin(conn)?;
    let lessons = repo::lessons_for_semester(&tx, semester_id)?;
    let cell: Vec<&Lesson> = lessons.iter().filter(|l| in_cell(l, day, &start, &end)).collect();
    if cell.is_empty() {
        return Err(AppError::key("errors.schedule.slot_not_found"));
    }
    let mut changed = 0;
    for lesson in cell {
        let mut current = lesson.faculty_ids.clone();
        current.sort_unstable();
        if current == wanted {
            continue;
        }
        let mut input = input_from(lesson);
        input.faculty_ids = wanted.clone();
        repo::update_lesson(&tx, lesson.id, &input)?;
        changed += 1;
    }
    commit(tx)?;
    if changed > 0 {
        logger::info("schedule", format!("факультеты слота {slot} (день {day}) изменены: {wanted:?}"));
    }
    Ok(())
}

/// Меняет лимит преподавателей у всех пар слота. Лимит не может быть меньше уже назначенных.
pub fn set_slot_teacher_count(conn: &Connection, semester_id: i64, day: i64, slot: &str, teacher_count: i64) -> AppResult<ScheduleOutcome> {
    if teacher_count < 1 {
        return Ok(ScheduleOutcome::TeacherCountInvalid);
    }
    let Some((start, end)) = parse_slot(slot) else { return Ok(ScheduleOutcome::InvalidTime) };
    check_day(day)?;

    let tx = repo::begin(conn)?;
    let lessons = repo::lessons_for_semester(&tx, semester_id)?;
    let cell: Vec<&Lesson> = lessons.iter().filter(|l| in_cell(l, day, &start, &end)).collect();
    if cell.is_empty() {
        return Ok(ScheduleOutcome::SlotNotFound);
    }
    let assigned: BTreeSet<i64> = cell.iter().flat_map(|l| l.teacher_ids.iter().copied()).collect();
    if (assigned.len() as i64) > teacher_count {
        return Ok(ScheduleOutcome::TeacherCountBelowAssigned { assigned: assigned.len() as i64 });
    }
    for lesson in cell {
        if lesson.teacher_count == Some(teacher_count) {
            continue;
        }
        let mut input = input_from(lesson);
        input.teacher_count = Some(teacher_count);
        repo::update_lesson(&tx, lesson.id, &input)?;
    }
    commit(tx)?;
    logger::info("schedule", format!("лимит слота {slot} (день {day}) изменён на {teacher_count}"));
    Ok(ScheduleOutcome::Ok)
}

/// Снимает всех преподавателей со слота. Пары остаются заглушками с тем же лимитом и факультетами.
pub fn clear_slot_assignments(conn: &Connection, semester_id: i64, day: i64, slot: &str) -> AppResult<()> {
    let (start, end) = parse_slot(slot).ok_or_else(|| AppError::key("errors.schedule.slot_not_found"))?;
    check_day(day)?;

    let tx = repo::begin(conn)?;
    let lessons = repo::lessons_for_semester(&tx, semester_id)?;
    let cell: Vec<&Lesson> = lessons.iter().filter(|l| in_cell(l, day, &start, &end)).collect();
    if cell.is_empty() {
        return Err(AppError::key("errors.schedule.slot_not_found"));
    }
    let mut cleared = 0;
    for lesson in cell.iter().filter(|l| !l.teacher_ids.is_empty()) {
        cleared += lesson.teacher_ids.len();
        let mut input = input_from(lesson);
        input.teacher_ids.clear();
        repo::update_lesson(&tx, lesson.id, &input)?;
    }
    commit(tx)?;
    if cleared > 0 {
        logger::info("schedule", format!("очищены назначения слота {slot} (день {day}): назначений {cleared}"));
    }
    Ok(())
}

/// Удаляет слот целиком: все пары слота и их связи.
pub fn delete_slot(conn: &Connection, semester_id: i64, day: i64, slot: &str) -> AppResult<()> {
    let (start, end) = parse_slot(slot).ok_or_else(|| AppError::key("errors.schedule.slot_not_found"))?;
    check_day(day)?;

    let tx = repo::begin(conn)?;
    let lessons = repo::lessons_for_semester(&tx, semester_id)?;
    let cell: Vec<&Lesson> = lessons.iter().filter(|l| in_cell(l, day, &start, &end)).collect();
    if cell.is_empty() {
        return Err(AppError::key("errors.schedule.slot_not_found"));
    }
    let assignments: usize = cell.iter().map(|l| l.teacher_ids.len()).sum();
    for lesson in &cell {
        repo::delete_lesson(&tx, lesson.id)?;
    }
    commit(tx)?;
    logger::info("schedule", format!("удалён слот {slot} (день {day}): пар {}, назначений {assignments}", cell.len()));
    Ok(())
}

/// Создаёт пустой слот (пару без преподавателей) с лимитом.
///
/// Без признака `custom_time` допускаются только стандартные пары. В обоих режимах проверяются
/// формат, порядок времени, точный дубль и пересечение с другими слотами дня.
pub fn add_slot(
    conn: &Connection,
    semester_id: i64,
    day: i64,
    start: &str,
    end: &str,
    teacher_count: i64,
    custom_time: bool,
) -> AppResult<ScheduleOutcome> {
    check_day(day)?;
    if teacher_count < 1 {
        return Ok(ScheduleOutcome::TeacherCountInvalid);
    }
    let (Some(start_min), Some(end_min)) = (parse_time(start), parse_time(end)) else {
        return Ok(ScheduleOutcome::InvalidTime);
    };
    if start_min >= end_min {
        return Ok(ScheduleOutcome::StartNotBeforeEnd);
    }
    if !custom_time && !STANDARD_PAIRS.iter().any(|(s, e)| *s == start && *e == end) {
        return Ok(ScheduleOutcome::NotStandardPair);
    }

    let tx = repo::begin(conn)?;
    let lessons = repo::lessons_for_semester(&tx, semester_id)?;
    let day_lessons: Vec<&Lesson> = lessons.iter().filter(|l| l.day_of_week == day).collect();
    if day_lessons.iter().any(|l| in_cell(l, day, start, end)) {
        return Ok(ScheduleOutcome::SlotExists);
    }
    let overlapping = day_lessons
        .iter()
        .any(|l| overlaps(time_minutes(&l.start_time), time_minutes(&l.end_time), start_min, end_min));
    if overlapping {
        return Ok(ScheduleOutcome::SlotOverlaps);
    }

    let direction_id = repo::list_directions(&tx, false)?.first().map(|d| d.id).unwrap_or(0);
    if direction_id == 0 {
        return Ok(ScheduleOutcome::SelectDirection);
    }
    let input = LessonInput {
        semester_id,
        day_of_week: day,
        start_time: start.to_owned(),
        end_time: end.to_owned(),
        direction_id,
        teacher_count: Some(teacher_count),
        faculty_ids: Vec::new(),
        teacher_ids: Vec::new(),
    };
    let id = repo::create_lesson(&tx, &input)?;
    commit(tx)?;
    logger::info(
        "schedule",
        format!("добавлен слот id={id}: день {day}, {start}-{end}, макс. преподавателей {teacher_count}{}", if custom_time { " (нестандартное время)" } else { "" }),
    );
    Ok(ScheduleOutcome::Ok)
}

fn commit(tx: rusqlite::Transaction<'_>) -> AppResult<()> {
    tx.commit().map_err(|e| AppError::with_cause("errors.database.commit_failed", e))
}

fn input_from(lesson: &Lesson) -> LessonInput {
    LessonInput {
        semester_id: lesson.semester_id,
        day_of_week: lesson.day_of_week,
        start_time: lesson.start_time.clone(),
        end_time: lesson.end_time.clone(),
        direction_id: lesson.direction_id,
        teacher_count: lesson.teacher_count,
        faculty_ids: lesson.faculty_ids.clone(),
        teacher_ids: lesson.teacher_ids.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Teacher;
    use crate::storage::db;

    const DAY: i64 = 1;
    const SLOT: &str = "07:45-09:20";

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        conn
    }

    fn teacher(first: &str, last: &str, direction_id: i64) -> Teacher {
        Teacher {
            id: 0,
            first_name: first.to_owned(),
            last_name: last.to_owned(),
            middle_name: String::new(),
            direction_id,
            rate: 1.0,
            direction_name: String::new(),
            direction_archived: false,
            is_archived: false,
            is_guest: false,
            is_bound: false,
        }
    }

    /// Направление, два преподавателя и семестр; оба преподавателя привязаны автоматически.
    fn setup(conn: &Connection) -> (i64, i64, i64) {
        let direction = repo::save_direction(conn, None, "Футбол", &[]).unwrap();
        let first = repo::create_teacher(conn, &teacher("Иван", "Иванов", direction.id)).unwrap();
        let second = repo::create_teacher(conn, &teacher("Пётр", "Петров", direction.id)).unwrap();
        let semester = repo::create_semester(conn, "Осень", "2026-09-01", "2026-12-31").unwrap();
        (semester.id, first.id, second.id)
    }

    fn add(conn: &Connection, semester: i64, start: &str, end: &str, limit: i64) -> ScheduleOutcome {
        add_slot(conn, semester, DAY, start, end, limit, false).unwrap()
    }

    fn lesson_count(conn: &Connection, semester: i64) -> usize {
        repo::lessons_for_semester(conn, semester).unwrap().len()
    }

    #[test]
    fn parse_time_is_strict() {
        assert_eq!(parse_time("09:40"), Some(9 * 60 + 40));
        assert_eq!(parse_time("9:40"), None);
        assert_eq!(parse_time("9.40"), None);
        assert_eq!(parse_time("25:00"), None);
        assert_eq!(parse_time("+1:00"), None);
    }

    #[test]
    fn normalize_time_pads_hour() {
        assert_eq!(normalize_time("9:00"), "09:00");
        assert_eq!(normalize_time(" 09:00 "), "09:00");
        assert_eq!(normalize_time("9.40"), "9.40");
    }

    #[test]
    fn assign_creates_pair_then_reuses_blank() {
        let conn = connection();
        let (semester, first, _) = setup(&conn);
        assert!(matches!(add(&conn, semester, "07:45", "09:20", 2), ScheduleOutcome::Ok));

        // Пустая заглушка переиспользуется: новая пара не создаётся.
        assert!(matches!(toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap(), ScheduleOutcome::Ok));
        assert_eq!(lesson_count(&conn, semester), 1);
    }

    #[test]
    fn unassign_from_single_pair_keeps_blank() {
        let conn = connection();
        let (semester, first, _) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 2);
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();

        let lessons = repo::lessons_for_semester(&conn, semester).unwrap();
        assert_eq!(lessons.len(), 1);
        assert!(lessons[0].teacher_ids.is_empty());
        assert_eq!(lessons[0].teacher_count, Some(2));
    }

    #[test]
    fn unassign_from_one_of_several_pairs_deletes_pair() {
        let conn = connection();
        let (semester, first, second) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 2);
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        toggle_teacher(&conn, semester, DAY, SLOT, second).unwrap();
        assert_eq!(lesson_count(&conn, semester), 2);

        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        assert_eq!(lesson_count(&conn, semester), 1);
    }

    #[test]
    fn teacher_limit_is_enforced_by_unique_teachers() {
        let conn = connection();
        let (semester, first, second) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 1);
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();

        let outcome = toggle_teacher(&conn, semester, DAY, SLOT, second).unwrap();
        assert!(matches!(outcome, ScheduleOutcome::TeacherCountExceeded { max: 1 }));
    }

    #[test]
    fn unbound_teacher_cannot_be_assigned() {
        let conn = connection();
        let (semester, first, _) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 2);
        repo::delete_binding(&conn, semester, first).unwrap();

        let outcome = toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        assert!(matches!(outcome, ScheduleOutcome::TeacherNotBound));
    }

    #[test]
    fn archived_direction_blocks_assignment() {
        let conn = connection();
        let (semester, first, _) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 2);
        let direction_id = repo::get_teacher(&conn, first).unwrap().direction_id;
        repo::archive_direction(&conn, direction_id).unwrap();

        let outcome = toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        assert!(matches!(outcome, ScheduleOutcome::DirectionArchived));
    }

    #[test]
    fn add_slot_validates_time_and_overlaps() {
        let conn = connection();
        let (semester, _, _) = setup(&conn);

        assert!(matches!(add(&conn, semester, "9.40", "11:15", 2), ScheduleOutcome::InvalidTime));
        assert!(matches!(add(&conn, semester, "11:15", "09:40", 2), ScheduleOutcome::StartNotBeforeEnd));
        assert!(matches!(add(&conn, semester, "07:45", "09:20", 0), ScheduleOutcome::TeacherCountInvalid));
        assert!(matches!(add(&conn, semester, "10:00", "11:00", 2), ScheduleOutcome::NotStandardPair));
        assert!(matches!(add(&conn, semester, "07:45", "09:20", 2), ScheduleOutcome::Ok));
        assert!(matches!(add(&conn, semester, "07:45", "09:20", 2), ScheduleOutcome::SlotExists));
        assert!(matches!(
            add_slot(&conn, semester, DAY, "08:00", "10:00", 2, true).unwrap(),
            ScheduleOutcome::SlotOverlaps
        ));
        assert!(matches!(
            add_slot(&conn, semester, DAY, "10:00", "11:00", 2, true).unwrap(),
            ScheduleOutcome::Ok
        ));
    }

    #[test]
    fn unbind_removes_assignments_and_reports_counts() {
        let conn = connection();
        let (semester, first, _) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 2);
        add(&conn, semester, "09:40", "11:15", 2);
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        toggle_teacher(&conn, semester, DAY, "09:40-11:15", first).unwrap();

        let preview = preview_unbind(&conn, semester, first).unwrap();
        assert_eq!((preview.assignments, preview.deleted_pairs), (2, 0));

        let removed = unbind_teacher(&conn, semester, first).unwrap();
        assert_eq!((removed.assignments, removed.deleted_pairs), (2, 0));
        let view = build_view(&conn, semester).unwrap();
        assert!(view.teachers.iter().all(|t| t.id != first));
        assert!(view.days[0].slots.iter().all(|s| s.assignment_count == 0));
    }

    #[test]
    fn teacher_count_cannot_drop_below_assigned() {
        let conn = connection();
        let (semester, first, second) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 2);
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        toggle_teacher(&conn, semester, DAY, SLOT, second).unwrap();

        let outcome = set_slot_teacher_count(&conn, semester, DAY, SLOT, 1).unwrap();
        assert!(matches!(outcome, ScheduleOutcome::TeacherCountBelowAssigned { assigned: 2 }));
        let outcome = set_slot_teacher_count(&conn, semester, DAY, SLOT, 3).unwrap();
        assert!(matches!(outcome, ScheduleOutcome::Ok));
        let view = build_view(&conn, semester).unwrap();
        assert_eq!(view.days[0].slots[0].teacher_max, Some(3));
    }

    #[test]
    fn clear_assignments_keeps_pairs_with_limit_and_faculties() {
        let conn = connection();
        let (semester, first, _) = setup(&conn);
        let faculty = repo::create_faculty(&conn, "Экономики").unwrap();
        add(&conn, semester, "07:45", "09:20", 2);
        set_slot_faculties(&conn, semester, DAY, SLOT, &[faculty.id]).unwrap();
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();

        clear_slot_assignments(&conn, semester, DAY, SLOT).unwrap();
        let lessons = repo::lessons_for_semester(&conn, semester).unwrap();
        assert_eq!(lessons.len(), 1);
        assert!(lessons[0].teacher_ids.is_empty());
        assert_eq!(lessons[0].teacher_count, Some(2));
        assert_eq!(lessons[0].faculty_ids, vec![faculty.id]);
    }

    #[test]
    fn faculties_apply_to_all_pairs_of_slot() {
        let conn = connection();
        let (semester, first, second) = setup(&conn);
        let faculty = repo::create_faculty(&conn, "Экономики").unwrap();
        add(&conn, semester, "07:45", "09:20", 2);
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        toggle_teacher(&conn, semester, DAY, SLOT, second).unwrap();
        set_slot_faculties(&conn, semester, DAY, SLOT, &[faculty.id]).unwrap();

        let lessons = repo::lessons_for_semester(&conn, semester).unwrap();
        assert!(lessons.iter().all(|l| l.faculty_ids == vec![faculty.id]));
    }

    #[test]
    fn delete_slot_removes_all_pairs() {
        let conn = connection();
        let (semester, first, second) = setup(&conn);
        add(&conn, semester, "07:45", "09:20", 2);
        toggle_teacher(&conn, semester, DAY, SLOT, first).unwrap();
        toggle_teacher(&conn, semester, DAY, SLOT, second).unwrap();

        delete_slot(&conn, semester, DAY, SLOT).unwrap();
        assert_eq!(lesson_count(&conn, semester), 0);
    }
}
