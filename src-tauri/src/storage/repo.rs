//! SQL-доступ к данным. Записи не открывают транзакций сами: их оборачивает вызывающий код
//! (`schedule`, `save_direction`, `create_semester`), чтобы многошаговая операция была атомарной.

use std::collections::{BTreeSet, HashMap};

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};
use crate::logger;
use crate::models::{Direction, Faculty, Lesson, LessonInput, Semester, Teacher};

/// Ключ сортировки по-русски: без учёта регистра, «ё» как «е». Порядок SQLite по коду символа не подходит.
pub fn sort_key(value: &str) -> String {
    value.trim().to_lowercase().replace('ё', "е")
}

/// Название с пометкой для экрана экспорта: архивные направления и факультеты остаются видимыми.
pub fn archived_label(name: &str, archived: bool) -> String {
    if archived { format!("{name} (архив)") } else { name.to_owned() }
}

fn db_err(code: &'static str, e: rusqlite::Error) -> AppError {
    AppError::with_cause(code, e)
}

pub fn begin(conn: &Connection) -> AppResult<rusqlite::Transaction<'_>> {
    conn.unchecked_transaction().map_err(|e| db_err("errors.database.transaction_failed", e))
}

fn commit(tx: rusqlite::Transaction<'_>) -> AppResult<()> {
    tx.commit().map_err(|e| db_err("errors.database.commit_failed", e))
}

/// Выполняет UPDATE или DELETE. Если ни одна строка не затронута, возвращает `not_found`.
fn exec_checked(
    conn: &Connection,
    sql: &str,
    args: impl rusqlite::Params,
    not_found: &'static str,
    failed: &'static str,
) -> AppResult<()> {
    let affected = conn.execute(sql, args).map_err(|e| db_err(failed, e))?;
    if affected == 0 {
        return Err(AppError::key(not_found));
    }
    Ok(())
}

fn clean_name(name: &str, required: &'static str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::key(required));
    }
    Ok(trimmed.to_owned())
}

/// Ставка — положительная и кратная 0,25 (правило предметной области, `AGENTS.md`).
pub fn validate_rate(rate: f64) -> AppResult<()> {
    let quarters = rate * 4.0;
    if !rate.is_finite() || rate <= 0.0 || (quarters - quarters.round()).abs() > 1e-9 {
        return Err(AppError::key("errors.teachers.rate_invalid"));
    }
    Ok(())
}

fn validated_dates(start: &str, end: &str) -> AppResult<(String, String)> {
    let parse = |value: &str| {
        NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| AppError::key("errors.semesters.date_invalid"))
    };
    let (start, end) = (parse(start)?, parse(end)?);
    if end < start {
        return Err(AppError::key("errors.semesters.date_order"));
    }
    Ok((start.format("%Y-%m-%d").to_string(), end.format("%Y-%m-%d").to_string()))
}

// ---------- Направления ----------

fn direction_from_row(row: &Row<'_>) -> rusqlite::Result<Direction> {
    Ok(Direction { id: row.get(0)?, name: row.get(1)?, is_archived: row.get(2)? })
}

pub fn list_directions(conn: &Connection, archived: bool) -> AppResult<Vec<Direction>> {
    let mut stmt = conn
        .prepare("SELECT id, name, isArchived FROM directions WHERE isArchived = ?1")
        .map_err(|e| db_err("errors.directions.get_all_failed", e))?;
    let mut list: Vec<Direction> = stmt
        .query_map(params![archived], direction_from_row)
        .map_err(|e| db_err("errors.directions.get_all_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.directions.iterate_failed", e))?;
    list.sort_by_cached_key(|d| (sort_key(&d.name), d.id));
    Ok(list)
}

/// Создаёт или переименовывает направление и задаёт его преподавателей одной транзакцией.
/// `teacher_ids` — итоговый набор. Преподаватель из другого направления в него не попадает:
/// сначала его нужно открепить там. Преподаватели, которых в наборе нет, откреплялись бы в «без направления».
pub fn save_direction(conn: &Connection, id: Option<i64>, name: &str, teacher_ids: &[i64]) -> AppResult<Direction> {
    let name = clean_name(name, "errors.directions.name_required")?;
    let teacher_ids: BTreeSet<i64> = teacher_ids.iter().copied().collect();
    let tx = begin(conn)?;

    let direction = match id {
        Some(id) => {
            exec_checked(
                &tx,
                "UPDATE directions SET name = ?1 WHERE id = ?2",
                params![name, id],
                "errors.directions.not_found",
                "errors.directions.update_failed",
            )?;
            let is_archived = tx
                .query_row("SELECT isArchived FROM directions WHERE id = ?1", params![id], |row| row.get(0))
                .map_err(|e| db_err("errors.directions.get_all_failed", e))?;
            Direction { id, name: name.clone(), is_archived }
        }
        None => {
            tx.execute("INSERT INTO directions (name) VALUES (?1)", params![name])
                .map_err(|e| db_err("errors.directions.create_failed", e))?;
            Direction { id: tx.last_insert_rowid(), name: name.clone(), is_archived: false }
        }
    };

    let current: Vec<i64> = {
        let mut stmt = tx
            .prepare("SELECT id FROM teachers WHERE direction_id = ?1")
            .map_err(|e| db_err("errors.teachers.get_all_failed", e))?;
        stmt.query_map(params![direction.id], |row| row.get(0))
            .map_err(|e| db_err("errors.teachers.get_all_failed", e))?
            .collect::<Result<_, _>>()
            .map_err(|e| db_err("errors.teachers.iterate_failed", e))?
    };

    for teacher_id in &teacher_ids {
        let direction_of: Option<i64> = tx
            .query_row("SELECT COALESCE(direction_id, 0) FROM teachers WHERE id = ?1", params![teacher_id], |row| row.get(0))
            .optional()
            .map_err(|e| db_err("errors.teachers.get_failed", e))?;
        match direction_of {
            None => return Err(AppError::key("errors.teachers.not_found")),
            Some(other) if other != 0 && other != direction.id => {
                return Err(AppError::key("errors.directions.teacher_assigned_elsewhere"));
            }
            Some(_) => {}
        }
    }

    for teacher_id in current.iter().filter(|id| !teacher_ids.contains(*id)) {
        tx.execute("UPDATE teachers SET direction_id = NULL WHERE id = ?1", params![teacher_id])
            .map_err(|e| db_err("errors.teachers.update_failed", e))?;
    }
    for teacher_id in teacher_ids.iter().filter(|id| !current.contains(*id)) {
        tx.execute("UPDATE teachers SET direction_id = ?1 WHERE id = ?2", params![direction.id, teacher_id])
            .map_err(|e| db_err("errors.teachers.update_failed", e))?;
    }

    commit(tx)?;
    match id {
        Some(id) => logger::info(
            "directions",
            format!("направление id={id} сохранено: \"{}\", преподавателей {}", direction.name, teacher_ids.len()),
        ),
        None => logger::info(
            "directions",
            format!("создано направление id={} \"{}\", преподавателей {}", direction.id, direction.name, teacher_ids.len()),
        ),
    }
    Ok(direction)
}

pub fn archive_direction(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE directions SET isArchived = TRUE WHERE id = ?1",
        params![id],
        "errors.directions.not_found",
        "errors.directions.delete_failed",
    )?;
    logger::info("directions", format!("направление id={id} перенесено в архив"));
    Ok(())
}

pub fn restore_direction(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE directions SET isArchived = FALSE WHERE id = ?1",
        params![id],
        "errors.directions.not_found",
        "errors.directions.restore_failed",
    )?;
    logger::info("directions", format!("направление id={id} восстановлено из архива"));
    Ok(())
}

// ---------- Факультеты ----------

fn faculty_from_row(row: &Row<'_>) -> rusqlite::Result<Faculty> {
    Ok(Faculty { id: row.get(0)?, name: row.get(1)?, is_archived: row.get(2)? })
}

fn sorted_faculties(mut list: Vec<Faculty>) -> Vec<Faculty> {
    list.sort_by_cached_key(|f| (sort_key(&f.name), f.id));
    list
}

pub fn create_faculty(conn: &Connection, name: &str) -> AppResult<Faculty> {
    let name = clean_name(name, "errors.faculties.name_required")?;
    conn.execute("INSERT INTO faculties (name) VALUES (?1)", params![name])
        .map_err(|e| db_err("errors.faculties.create_failed", e))?;
    let id = conn.last_insert_rowid();
    logger::info("faculties", format!("создан факультет id={id} \"{name}\""));
    Ok(Faculty { id, name, is_archived: false })
}

pub fn list_faculties(conn: &Connection, archived: bool) -> AppResult<Vec<Faculty>> {
    let mut stmt = conn
        .prepare("SELECT id, name, isArchived FROM faculties WHERE isArchived = ?1")
        .map_err(|e| db_err("errors.faculties.get_all_failed", e))?;
    let list = stmt
        .query_map(params![archived], faculty_from_row)
        .map_err(|e| db_err("errors.faculties.get_all_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.faculties.iterate_failed", e))?;
    Ok(sorted_faculties(list))
}

/// Все факультеты, включая архивные: для подписей в расписании.
pub fn all_faculties(conn: &Connection) -> AppResult<Vec<Faculty>> {
    let mut stmt = conn
        .prepare("SELECT id, name, isArchived FROM faculties")
        .map_err(|e| db_err("errors.faculties.get_all_failed", e))?;
    let list = stmt
        .query_map([], faculty_from_row)
        .map_err(|e| db_err("errors.faculties.get_all_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.faculties.iterate_failed", e))?;
    Ok(sorted_faculties(list))
}

pub fn update_faculty(conn: &Connection, id: i64, name: &str) -> AppResult<()> {
    let name = clean_name(name, "errors.faculties.name_required")?;
    exec_checked(
        conn,
        "UPDATE faculties SET name = ?1 WHERE id = ?2",
        params![name, id],
        "errors.faculties.not_found",
        "errors.faculties.update_failed",
    )?;
    logger::info("faculties", format!("факультет id={id} переименован в \"{name}\""));
    Ok(())
}

pub fn archive_faculty(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE faculties SET isArchived = TRUE WHERE id = ?1",
        params![id],
        "errors.faculties.not_found",
        "errors.faculties.delete_failed",
    )?;
    logger::info("faculties", format!("факультет id={id} перенесён в архив"));
    Ok(())
}

pub fn restore_faculty(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE faculties SET isArchived = FALSE WHERE id = ?1",
        params![id],
        "errors.faculties.not_found",
        "errors.faculties.restore_failed",
    )?;
    logger::info("faculties", format!("факультет id={id} восстановлен из архива"));
    Ok(())
}

// ---------- Преподаватели ----------

const TEACHER_COLUMNS: &str = "t.id, t.first_name, t.last_name, COALESCE(t.middle_name, ''), COALESCE(t.direction_id, 0), t.rate,
       COALESCE(d.name, ''), COALESCE(d.isArchived, FALSE), t.isArchived, t.isGuest";

const TEACHER_FROM: &str = "FROM teachers t LEFT JOIN directions d ON t.direction_id = d.id";

fn teacher_from_row(row: &Row<'_>) -> rusqlite::Result<Teacher> {
    Ok(Teacher {
        id: row.get(0)?,
        first_name: row.get(1)?,
        last_name: row.get(2)?,
        middle_name: row.get(3)?,
        direction_id: row.get(4)?,
        rate: row.get(5)?,
        direction_name: row.get(6)?,
        direction_archived: row.get(7)?,
        is_archived: row.get(8)?,
        is_guest: row.get(9)?,
        is_bound: false,
    })
}

fn sorted_teachers(mut list: Vec<Teacher>) -> Vec<Teacher> {
    list.sort_by_cached_key(|t| (sort_key(&t.last_name), sort_key(&t.first_name), t.id));
    list
}

/// 0 в API означает «без направления», в базе это NULL.
fn direction_param(direction_id: i64) -> Option<i64> {
    (direction_id != 0).then_some(direction_id)
}

/// Проверяет поля преподавателя и возвращает очищенные ФИО.
fn validated_teacher(conn: &Connection, input: &Teacher) -> AppResult<(String, String, String)> {
    let first = clean_name(&input.first_name, "errors.teachers.name_required")?;
    let last = clean_name(&input.last_name, "errors.teachers.name_required")?;
    validate_rate(input.rate)?;
    if input.direction_id != 0 {
        let exists: Option<i64> = conn
            .query_row("SELECT id FROM directions WHERE id = ?1", params![input.direction_id], |row| row.get(0))
            .optional()
            .map_err(|e| db_err("errors.directions.get_all_failed", e))?;
        if exists.is_none() {
            return Err(AppError::key("errors.teachers.direction_not_found"));
        }
    }
    Ok((first, last, input.middle_name.trim().to_owned()))
}

pub fn get_teacher(conn: &Connection, id: i64) -> AppResult<Teacher> {
    let sql = format!("SELECT {TEACHER_COLUMNS} {TEACHER_FROM} WHERE t.id = ?1");
    conn.query_row(&sql, params![id], teacher_from_row)
        .optional()
        .map_err(|e| db_err("errors.teachers.get_failed", e))?
        .ok_or_else(|| AppError::key("errors.teachers.not_found"))
}

pub fn create_teacher(conn: &Connection, input: &Teacher) -> AppResult<Teacher> {
    let (first, last, middle) = validated_teacher(conn, input)?;
    conn.execute(
        "INSERT INTO teachers (first_name, last_name, middle_name, direction_id, rate, isGuest)
         VALUES (?1, ?2, ?3, ?4, ?5, FALSE)",
        params![first, last, middle, direction_param(input.direction_id), input.rate],
    )
    .map_err(|e| db_err("errors.teachers.create_failed", e))?;
    let id = conn.last_insert_rowid();
    logger::info("teachers", format!("создан преподаватель id={id} {last} {first}"));
    get_teacher(conn, id)
}

pub fn list_teachers(conn: &Connection, archived: bool) -> AppResult<Vec<Teacher>> {
    let sql = format!("SELECT {TEACHER_COLUMNS} {TEACHER_FROM} WHERE t.isArchived = ?1");
    let mut stmt = conn.prepare(&sql).map_err(|e| db_err("errors.teachers.get_all_failed", e))?;
    let list = stmt
        .query_map(params![archived], teacher_from_row)
        .map_err(|e| db_err("errors.teachers.get_all_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.teachers.iterate_failed", e))?;
    Ok(sorted_teachers(list))
}

pub fn update_teacher(conn: &Connection, input: &Teacher) -> AppResult<()> {
    let (first, last, middle) = validated_teacher(conn, input)?;
    exec_checked(
        conn,
        "UPDATE teachers SET first_name = ?1, last_name = ?2, middle_name = ?3, direction_id = ?4, rate = ?5 WHERE id = ?6",
        params![first, last, middle, direction_param(input.direction_id), input.rate, input.id],
        "errors.teachers.not_found",
        "errors.teachers.update_failed",
    )?;
    logger::info("teachers", format!("преподаватель id={} обновлён: {last} {first}", input.id));
    Ok(())
}

pub fn archive_teacher(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE teachers SET isArchived = TRUE WHERE id = ?1",
        params![id],
        "errors.teachers.not_found",
        "errors.teachers.delete_failed",
    )?;
    logger::info("teachers", format!("преподаватель id={id} перенесён в архив"));
    Ok(())
}

pub fn restore_teacher(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE teachers SET isArchived = FALSE WHERE id = ?1",
        params![id],
        "errors.teachers.not_found",
        "errors.teachers.restore_failed",
    )?;
    logger::info("teachers", format!("преподаватель id={id} восстановлен из архива"));
    Ok(())
}

/// Направление преподавателя: `(direction_id, архивно ли направление)`. `None` — направления нет или оно удалено.
pub fn teacher_direction(conn: &Connection, teacher_id: i64) -> AppResult<(i64, Option<bool>)> {
    let row: Option<(i64, Option<bool>)> = conn
        .query_row(
            "SELECT COALESCE(t.direction_id, 0), d.isArchived FROM teachers t
             LEFT JOIN directions d ON d.id = t.direction_id WHERE t.id = ?1",
            params![teacher_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| db_err("errors.teachers.get_failed", e))?;
    row.ok_or_else(|| AppError::key("errors.teachers.not_found"))
}

// ---------- Семестры ----------

fn semester_from_row(row: &Row<'_>) -> rusqlite::Result<Semester> {
    Ok(Semester {
        id: row.get(0)?,
        name: row.get(1)?,
        start_date: row.get(2)?,
        end_date: row.get(3)?,
        is_archived: row.get(4)?,
    })
}

pub fn create_semester(conn: &Connection, name: &str, start: &str, end: &str) -> AppResult<Semester> {
    let name = clean_name(name, "errors.semesters.name_required")?;
    let (start, end) = validated_dates(start, end)?;
    let tx = begin(conn)?;
    tx.execute(
        "INSERT INTO semesters (name, start_date, end_date) VALUES (?1, ?2, ?3)",
        params![name, start, end],
    )
    .map_err(|e| db_err("errors.semesters.create_failed", e))?;
    let id = tx.last_insert_rowid();
    // Новый семестр получает всех негостевых неархивных преподавателей.
    tx.execute(
        "INSERT INTO semester_teachers (semester_id, teacher_id)
         SELECT ?1, t.id FROM teachers t WHERE t.isArchived = FALSE AND t.isGuest = FALSE",
        params![id],
    )
    .map_err(|e| db_err("errors.semesters.bind_teachers_failed", e))?;
    commit(tx)?;
    logger::info("semesters", format!("создан семестр id={id} \"{name}\" ({start} — {end})"));
    Ok(Semester { id, name, start_date: start, end_date: end, is_archived: false })
}

pub fn get_semester(conn: &Connection, id: i64) -> AppResult<Semester> {
    conn.query_row(
        "SELECT id, name, start_date, end_date, isArchived FROM semesters WHERE id = ?1",
        params![id],
        semester_from_row,
    )
    .optional()
    .map_err(|e| db_err("errors.semesters.get_failed", e))?
    .ok_or_else(|| AppError::key("errors.semesters.not_found"))
}

pub fn list_semesters(conn: &Connection, archived: bool) -> AppResult<Vec<Semester>> {
    let mut stmt = conn
        .prepare("SELECT id, name, start_date, end_date, isArchived FROM semesters WHERE isArchived = ?1 ORDER BY start_date DESC, id DESC")
        .map_err(|e| db_err("errors.semesters.get_all_failed", e))?;
    let rows = stmt
        .query_map(params![archived], semester_from_row)
        .map_err(|e| db_err("errors.semesters.get_all_failed", e))?;
    rows.collect::<Result<_, _>>().map_err(|e| db_err("errors.semesters.iterate_failed", e))
}

pub fn update_semester(conn: &Connection, id: i64, name: &str, start: &str, end: &str) -> AppResult<()> {
    let name = clean_name(name, "errors.semesters.name_required")?;
    let (start, end) = validated_dates(start, end)?;
    exec_checked(
        conn,
        "UPDATE semesters SET name = ?1, start_date = ?2, end_date = ?3 WHERE id = ?4",
        params![name, start, end, id],
        "errors.semesters.not_found",
        "errors.semesters.update_failed",
    )?;
    logger::info("semesters", format!("семестр id={id} обновлён: \"{name}\" ({start} — {end})"));
    Ok(())
}

pub fn archive_semester(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE semesters SET isArchived = TRUE WHERE id = ?1",
        params![id],
        "errors.semesters.not_found",
        "errors.semesters.delete_failed",
    )?;
    logger::info("semesters", format!("семестр id={id} перенесён в архив"));
    Ok(())
}

pub fn restore_semester(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE semesters SET isArchived = FALSE WHERE id = ?1",
        params![id],
        "errors.semesters.not_found",
        "errors.semesters.restore_failed",
    )?;
    logger::info("semesters", format!("семестр id={id} восстановлен из архива"));
    Ok(())
}

/// Все преподаватели, доступные для семестра, с признаком привязки. Архивные идут вперемешку по ФИО.
pub fn teachers_for_semester(conn: &Connection, semester_id: i64) -> AppResult<Vec<Teacher>> {
    let sql = format!(
        "SELECT {TEACHER_COLUMNS}, CASE WHEN st.teacher_id IS NOT NULL THEN 1 ELSE 0 END
         {TEACHER_FROM}
         LEFT JOIN semester_teachers st ON t.id = st.teacher_id AND st.semester_id = ?1"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| db_err("errors.semesters.get_teachers_failed", e))?;
    let list = stmt
        .query_map(params![semester_id], |row| {
            let mut teacher = teacher_from_row(row)?;
            teacher.is_bound = row.get::<_, i64>(10)? == 1;
            Ok(teacher)
        })
        .map_err(|e| db_err("errors.semesters.get_teachers_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.teachers.iterate_failed", e))?;
    Ok(sorted_teachers(list))
}

pub fn is_bound(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM semester_teachers WHERE semester_id = ?1 AND teacher_id = ?2)",
        params![semester_id, teacher_id],
        |row| row.get(0),
    )
    .map_err(|e| db_err("errors.semesters.get_teachers_failed", e))
}

pub fn bind_teacher(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<()> {
    get_semester(conn, semester_id)?;

    let teacher_is_guest: bool = conn
        .query_row("SELECT isGuest FROM teachers WHERE id = ?1", params![teacher_id], |row| row.get(0))
        .optional()
        .map_err(|e| db_err("errors.teachers.get_failed", e))?
        .ok_or_else(|| AppError::key("errors.teachers.not_found"))?;

    if is_bound(conn, semester_id, teacher_id)? {
        return Ok(());
    }

    // Гостевой преподаватель может быть привязан только к одному семестру.
    if teacher_is_guest {
        let other: Option<i64> = conn
            .query_row(
                "SELECT semester_id FROM semester_teachers WHERE teacher_id = ?1 LIMIT 1",
                params![teacher_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| db_err("errors.semesters.bind_teacher_failed", e))?;
        if other.is_some_and(|sid| sid != semester_id) {
            return Err(AppError::key("errors.semesters.guest_already_assigned"));
        }
    }

    conn.execute(
        "INSERT INTO semester_teachers (semester_id, teacher_id) VALUES (?1, ?2)",
        params![semester_id, teacher_id],
    )
    .map_err(|e| db_err("errors.semesters.bind_teacher_failed", e))?;
    logger::info("semesters", format!("преподаватель id={teacher_id} привязан к семестру id={semester_id}"));
    Ok(())
}

pub fn delete_binding(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "DELETE FROM semester_teachers WHERE semester_id = ?1 AND teacher_id = ?2",
        params![semester_id, teacher_id],
        "errors.semesters.bind_not_found",
        "errors.semesters.unbind_teacher_failed",
    )
}

// ---------- Занятия ----------

/// Загружает занятия семестра со связями. Связи читаются тремя запросами, без запроса на каждую пару.
pub fn lessons_for_semester(conn: &Connection, semester_id: i64) -> AppResult<Vec<Lesson>> {
    let mut lessons: Vec<Lesson> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, semester_id, day_of_week, start_time, end_time, direction_id, teacher_count
                 FROM lessons WHERE semester_id = ?1 ORDER BY day_of_week, start_time, end_time, id",
            )
            .map_err(|e| db_err("errors.lessons.get_all_failed", e))?;
        let rows = stmt
            .query_map(params![semester_id], |row| {
                Ok(Lesson {
                    id: row.get(0)?,
                    semester_id: row.get(1)?,
                    day_of_week: row.get(2)?,
                    start_time: row.get(3)?,
                    end_time: row.get(4)?,
                    direction_id: row.get(5)?,
                    teacher_count: row.get(6)?,
                    faculty_ids: Vec::new(),
                    faculty_names: Vec::new(),
                    teacher_ids: Vec::new(),
                    teacher_names: Vec::new(),
                })
            })
            .map_err(|e| db_err("errors.lessons.get_all_failed", e))?;
        rows.collect::<Result<_, _>>().map_err(|e| db_err("errors.lessons.iterate_failed", e))?
    };

    let mut faculties: HashMap<i64, Vec<(i64, String, bool)>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT lf.lesson_id, f.id, f.name, f.isArchived FROM lesson_faculties lf
                 JOIN lessons l ON l.id = lf.lesson_id
                 JOIN faculties f ON f.id = lf.faculty_id
                 WHERE l.semester_id = ?1",
            )
            .map_err(|e| db_err("errors.lessons.get_faculties_failed", e))?;
        let rows = stmt
            .query_map(params![semester_id], |row| {
                Ok((row.get::<_, i64>(0)?, (row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, bool>(3)?)))
            })
            .map_err(|e| db_err("errors.lessons.get_faculties_failed", e))?;
        for row in rows {
            let (lesson_id, faculty) = row.map_err(|e| db_err("errors.lessons.get_faculties_failed", e))?;
            faculties.entry(lesson_id).or_default().push(faculty);
        }
    }

    let mut teachers: HashMap<i64, Vec<(i64, String, String)>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT lt.lesson_id, t.id, t.first_name, t.last_name FROM lesson_teachers lt
                 JOIN lessons l ON l.id = lt.lesson_id
                 JOIN teachers t ON t.id = lt.teacher_id
                 WHERE l.semester_id = ?1",
            )
            .map_err(|e| db_err("errors.lessons.get_teachers_failed", e))?;
        let rows = stmt
            .query_map(params![semester_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    (row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?),
                ))
            })
            .map_err(|e| db_err("errors.lessons.get_teachers_failed", e))?;
        for row in rows {
            let (lesson_id, teacher) = row.map_err(|e| db_err("errors.lessons.get_teachers_failed", e))?;
            teachers.entry(lesson_id).or_default().push(teacher);
        }
    }

    for lesson in &mut lessons {
        if let Some(mut list) = faculties.remove(&lesson.id) {
            list.sort_by_cached_key(|(id, name, _)| (sort_key(name), *id));
            lesson.faculty_ids = list.iter().map(|(id, _, _)| *id).collect();
            lesson.faculty_names = list.iter().map(|(_, name, archived)| archived_label(name, *archived)).collect();
        }
        if let Some(mut list) = teachers.remove(&lesson.id) {
            list.sort_by_cached_key(|(id, first, last)| (sort_key(last), sort_key(first), *id));
            lesson.teacher_ids = list.iter().map(|(id, _, _)| *id).collect();
            lesson.teacher_names = list.iter().map(|(_, first, last)| format!("{first} {last}")).collect();
        }
    }
    Ok(lessons)
}

/// Запись связей пары. Вызывать внутри транзакции: при нарушении лимита транзакция откатывается.
fn write_links(conn: &Connection, lesson_id: i64, input: &LessonInput) -> AppResult<()> {
    if let Some(limit) = input.teacher_count && input.teacher_ids.len() as i64 > limit {
        return Err(AppError::key("errors.lessons.teacher_count_exceeded"));
    }

    for faculty_id in &input.faculty_ids {
        conn.execute(
            "INSERT INTO lesson_faculties (lesson_id, faculty_id) VALUES (?1, ?2)",
            params![lesson_id, faculty_id],
        )
        .map_err(|e| db_err("errors.lessons.create_faculty_link_failed", e))?;
    }

    for teacher_id in &input.teacher_ids {
        conn.execute(
            "INSERT INTO lesson_teachers (lesson_id, teacher_id) VALUES (?1, ?2)",
            params![lesson_id, teacher_id],
        )
        .map_err(|e| db_err("errors.lessons.create_teacher_link_failed", e))?;
    }
    Ok(())
}

/// Создаёт пару со связями. Вызывать внутри транзакции.
pub fn create_lesson(conn: &Connection, input: &LessonInput) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO lessons (semester_id, day_of_week, start_time, end_time, direction_id, teacher_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            input.semester_id,
            input.day_of_week,
            input.start_time,
            input.end_time,
            input.direction_id,
            input.teacher_count
        ],
    )
    .map_err(|e| db_err("errors.lessons.create_failed", e))?;
    let id = conn.last_insert_rowid();
    write_links(conn, id, input)?;
    Ok(id)
}

/// Заменяет пару и её связи. Вызывать внутри транзакции.
pub fn update_lesson(conn: &Connection, id: i64, input: &LessonInput) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE lessons SET semester_id = ?1, day_of_week = ?2, start_time = ?3, end_time = ?4,
                            direction_id = ?5, teacher_count = ?6
         WHERE id = ?7",
        params![
            input.semester_id,
            input.day_of_week,
            input.start_time,
            input.end_time,
            input.direction_id,
            input.teacher_count,
            id
        ],
        "errors.lessons.not_found",
        "errors.lessons.update_failed",
    )?;
    conn.execute("DELETE FROM lesson_faculties WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_faculty_links_failed", e))?;
    conn.execute("DELETE FROM lesson_teachers WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_teacher_links_failed", e))?;
    write_links(conn, id, input)
}

/// Удаляет пару и её связи. Вызывать внутри транзакции.
pub fn delete_lesson(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM lesson_faculties WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_faculty_links_failed", e))?;
    conn.execute("DELETE FROM lesson_teachers WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_teacher_links_failed", e))?;
    exec_checked(
        conn,
        "DELETE FROM lessons WHERE id = ?1",
        params![id],
        "errors.lessons.not_found",
        "errors.lessons.delete_failed",
    )
}
