//! Правила расписания. Фронтенд только отображает готовую модель и вызывает операции.
//! Слот — строка расписания «HH:MM-HH:MM» (день + интервал). Пары слота — занятия с этим днём и интервалом.

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use ts_rs::TS;

use crate::error::{AppError, AppResult};
use crate::logger;
use crate::models::{Direction, Faculty, Lesson, LessonInput};
use crate::storage::repo;

/// Нормализует время: «9:40» → «09:40».
pub fn normalize_time(value: &str) -> String {
    let mut parts = value.split(':');
    match (parts.next(), parts.next()) {
        (Some(h), Some(m)) => format!("{:0>2}:{:0>2}", h.trim(), m.trim()),
        _ => value.trim().to_owned(),
    }
}

fn parse_slot(slot: &str) -> Option<(String, String)> {
    let (start, end) = slot.split_once('-')?;
    Some((normalize_time(start), normalize_time(end)))
}

fn minutes(value: &str) -> u32 {
    let mut parts = value.split(':');
    let h: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let m: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    h * 60 + m
}

fn cell_lessons<'a>(lessons: &'a [Lesson], day: i64, slot: &str) -> Vec<&'a Lesson> {
    let Some((start, end)) = parse_slot(slot) else { return Vec::new() };
    lessons
        .iter()
        .filter(|l| l.day_of_week == day && l.start_time == start && l.end_time == end)
        .collect()
}

/// Результат операции расписания. Ошибки — коды, тексты берёт фронтенд из локалей.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ScheduleOutcome {
    Ok,
    TeacherCountExceeded { max: i64 },
    TeacherCountRequired,
    SlotExists,
    SelectDirection,
}

/// Модель расписания для экрана: слоты по дням и ячейки, уже посчитанные на бэкенде.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "generated/")]
#[serde(rename_all = "camelCase")]
pub struct ScheduleView {
    pub days: Vec<DayView>,
    pub teachers: Vec<TeacherView>,
    pub faculties: Vec<Faculty>,
    pub directions: Vec<Direction>,
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
    /// Лимит по `teacher_count` (максимум по парам); `None`, если пар с лимитом нет.
    pub teacher_max: Option<i64>,
    /// Число уникальных назначенных преподавателей слота.
    pub teacher_current: usize,
    /// Преподаватели, назначенные в слоте (по id).
    pub assigned_teacher_ids: Vec<i64>,
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
    pub rate: f64,
    /// Занятий в семестре (назначено пар с этим преподавателем).
    pub assigned: usize,
    /// Целевое число занятий: `rate * 12`.
    pub target: usize,
}

/// Полная ставка — 12 занятий в неделю (правило предметной области).
const FULL_RATE_LESSONS: f64 = 12.0;

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
        faculties: repo::list_faculties(conn, false)?,
        directions: repo::list_directions(conn, false)?,
    })
}

fn slots_for_day(lessons: &[Lesson], day: i64) -> Vec<SlotView> {
    // Уникальные слоты, отсортированные по времени начала, затем конца.
    let mut keyed: BTreeMap<(u32, u32, String), (String, String)> = BTreeMap::new();
    for l in lessons.iter().filter(|l| l.day_of_week == day) {
        let key = (minutes(&l.start_time), minutes(&l.end_time), format!("{}-{}", l.start_time, l.end_time));
        keyed.entry(key).or_insert_with(|| (l.start_time.clone(), l.end_time.clone()));
    }

    keyed
        .into_iter()
        .map(|((_, _, slot), (start_time, end_time))| {
            let pairs = cell_lessons(lessons, day, &slot);
            let mut faculty_ids: Vec<i64> = pairs.iter().flat_map(|l| l.faculty_ids.iter().copied()).collect();
            faculty_ids.sort_unstable();
            faculty_ids.dedup();
            let mut assigned: Vec<i64> = pairs.iter().flat_map(|l| l.teacher_ids.iter().copied()).collect();
            assigned.sort_unstable();
            assigned.dedup();
            SlotView {
                slot,
                start_time,
                end_time,
                faculty_ids,
                teacher_max: pairs.iter().filter_map(|l| l.teacher_count).max(),
                teacher_current: assigned.len(),
                assigned_teacher_ids: assigned,
            }
        })
        .collect()
}

/// Направление для новой пары: у преподавателя своё, иначе первое доступное направление (0, если нет ни одного).
fn direction_for_teacher(conn: &Connection, teacher_id: i64) -> AppResult<i64> {
    let own: Option<i64> = conn
        .query_row("SELECT direction_id FROM teachers WHERE id = ?1", [teacher_id], |row| row.get(0))
        .optional()
        .map_err(|e| AppError::with_cause("errors.teachers.get_failed", e))?
        .filter(|id| *id != 0);
    match own {
        Some(id) => Ok(id),
        None => Ok(repo::list_directions(conn, false)?.first().map(|d| d.id).unwrap_or(0)),
    }
}

/// Переключает преподавателя в ячейке слота.
///
/// Снятие: если преподаватель не последний в паре — убираем только его; если пара единственная
/// в слоте — оставляем пустую заглушку с тем же лимитом; иначе удаляем пару целиком.
/// Назначение: переиспользуем пустую заглушку слота, иначе создаём новую пару.
pub fn toggle_teacher(conn: &Connection, semester_id: i64, day: i64, slot: &str, teacher_id: i64) -> AppResult<ScheduleOutcome> {
    let lessons = repo::lessons_for_semester(conn, semester_id)?;
    let Some((start, end)) = parse_slot(slot) else { return Ok(ScheduleOutcome::Ok) };
    let pairs = cell_lessons(&lessons, day, slot);

    if let Some(existing) = pairs.iter().find(|l| l.teacher_ids.contains(&teacher_id)) {
        let mut input = input_from(existing);
        if existing.teacher_ids.len() > 1 {
            input.teacher_ids.retain(|id| *id != teacher_id);
            repo::update_lesson(conn, existing.id, &input)?;
            logger::info("schedule", format!("снят преподаватель id={teacher_id} с пары id={}", existing.id));
        } else if pairs.len() <= 1 {
            input.teacher_ids.clear();
            repo::update_lesson(conn, existing.id, &input)?;
            logger::info("schedule", format!("пара id={} очищена до заглушки (снят id={teacher_id})", existing.id));
        } else {
            repo::delete_lesson(conn, existing.id)?;
            logger::info("schedule", format!("удалена пара id={} (снят id={teacher_id})", existing.id));
        }
        return Ok(ScheduleOutcome::Ok);
    }

    let direction_id = direction_for_teacher(conn, teacher_id)?;
    if direction_id == 0 {
        return Ok(ScheduleOutcome::SelectDirection);
    }

    let max = pairs.iter().filter_map(|l| l.teacher_count).max();
    let current = {
        let mut ids: Vec<i64> = pairs.iter().flat_map(|l| l.teacher_ids.iter().copied()).collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len() as i64
    };
    if let Some(max) = max {
        if current + 1 > max {
            logger::warn("schedule", format!("лимит преподавателей слота {slot} (день {day}) исчерпан: max={max}"));
            return Ok(ScheduleOutcome::TeacherCountExceeded { max });
        }
    }

    let faculty_ids = {
        let mut ids: Vec<i64> = pairs.iter().flat_map(|l| l.faculty_ids.iter().copied()).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };
    let blank = pairs.iter().find(|l| l.teacher_ids.is_empty()).map(|l| (*l).clone());
    let Some(teacher_count) = blank.as_ref().and_then(|l| l.teacher_count).or(max) else {
        return Ok(ScheduleOutcome::TeacherCountRequired);
    };

    let input = LessonInput {
        semester_id,
        day_of_week: day,
        start_time: start,
        end_time: end,
        direction_id,
        teacher_count: Some(teacher_count),
        faculty_ids,
        teacher_ids: vec![teacher_id],
    };
    match blank {
        Some(lesson) => {
            repo::update_lesson(conn, lesson.id, &input)?;
            logger::info("schedule", format!("назначен преподаватель id={teacher_id} в заглушку пары id={}", lesson.id));
        }
        None => {
            let id = repo::create_lesson(conn, &input)?;
            logger::info("schedule", format!("создана пара id={id}: день {day}, {slot}, преподаватель id={teacher_id}"));
        }
    }
    Ok(ScheduleOutcome::Ok)
}

/// Заменяет факультеты строки во всех парах слота.
pub fn set_slot_faculties(conn: &Connection, semester_id: i64, day: i64, slot: &str, faculty_ids: &[i64]) -> AppResult<()> {
    let lessons = repo::lessons_for_semester(conn, semester_id)?;
    for lesson in cell_lessons(&lessons, day, slot) {
        let mut input = input_from(lesson);
        input.faculty_ids = faculty_ids.to_vec();
        repo::update_lesson(conn, lesson.id, &input)?;
    }
    logger::info("schedule", format!("факультеты слота {slot} (день {day}) изменены: {faculty_ids:?}"));
    Ok(())
}

/// Удаляет слот: сначала очищает факультеты, затем удаляет все пары слота.
pub fn delete_slot(conn: &Connection, semester_id: i64, day: i64, slot: &str) -> AppResult<()> {
    set_slot_faculties(conn, semester_id, day, slot, &[])?;
    let lessons = repo::lessons_for_semester(conn, semester_id)?;
    let pairs = cell_lessons(&lessons, day, slot);
    for lesson in &pairs {
        repo::delete_lesson(conn, lesson.id)?;
    }
    logger::info("schedule", format!("удалён слот {slot} (день {day}): пар {}", pairs.len()));
    Ok(())
}

/// Создаёт пустой слот (пару без преподавателей) с лимитом.
pub fn add_slot(conn: &Connection, semester_id: i64, day: i64, start: &str, end: &str, teacher_count: i64) -> AppResult<ScheduleOutcome> {
    let lessons = repo::lessons_for_semester(conn, semester_id)?;
    let (start, end) = (normalize_time(start), normalize_time(end));
    let slot = format!("{start}-{end}");
    if lessons.iter().any(|l| l.day_of_week == day && l.start_time == start && l.end_time == end) {
        return Ok(ScheduleOutcome::SlotExists);
    }
    let direction_id = repo::list_directions(conn, false)?.first().map(|d| d.id).unwrap_or(0);
    if direction_id == 0 {
        return Ok(ScheduleOutcome::SelectDirection);
    }
    let input = LessonInput {
        semester_id,
        day_of_week: day,
        start_time: start.clone(),
        end_time: end.clone(),
        direction_id,
        teacher_count: Some(teacher_count),
        faculty_ids: Vec::new(),
        teacher_ids: Vec::new(),
    };
    let id = repo::create_lesson(conn, &input)?;
    logger::info("schedule", format!("добавлен слот id={id}: день {day}, {slot}, макс. преподавателей {teacher_count}"));
    Ok(ScheduleOutcome::Ok)
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

