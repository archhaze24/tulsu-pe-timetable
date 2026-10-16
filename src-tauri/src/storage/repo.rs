use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};
use crate::logger;
use crate::models::{Direction, Faculty, Lesson, LessonInput, Semester, Teacher};

fn not_found(key: &str) -> AppError {
    AppError::key(key)
}

fn db_err(key: &str, e: rusqlite::Error) -> AppError {
    AppError::with_cause(key, e)
}

/// Выполняет UPDATE или DELETE. Если ни одна строка не затронута, возвращает `not_found`.
/// Успешное действие пишется в журнал один раз здесь; ошибки записывает вызывающий код через `respond`.
fn exec_checked(conn: &Connection, sql: &str, args: impl rusqlite::Params, action: &str, failed: &str, not_found_key: &str) -> AppResult<()> {
    let affected = conn.execute(sql, args).map_err(|e| db_err(failed, e))?;
    if affected == 0 {
        return Err(not_found(not_found_key));
    }
    if !action.is_empty() {
        logger::info("storage", action);
    }
    Ok(())
}

// ---------- Направления ----------

pub fn create_direction(conn: &Connection, name: &str) -> AppResult<Direction> {
    conn.execute("INSERT INTO directions (name) VALUES (?1)", params![name])
        .map_err(|e| db_err("errors.directions.create_failed", e))?;
    let id = conn.last_insert_rowid();
    logger::info("directions", format!("создано направление id={id} \"{name}\""));
    Ok(Direction { id, name: name.to_owned(), is_archived: false })
}

pub fn list_directions(conn: &Connection, archived: bool) -> AppResult<Vec<Direction>> {
    let mut stmt = conn
        .prepare("SELECT id, name, isArchived FROM directions WHERE isArchived = ?1 ORDER BY name")
        .map_err(|e| db_err("errors.directions.get_all_failed", e))?;
    let rows = stmt
        .query_map(params![archived], |row| {
            Ok(Direction { id: row.get(0)?, name: row.get(1)?, is_archived: row.get(2)? })
        })
        .map_err(|e| db_err("errors.directions.get_all_failed", e))?;
    rows.collect::<Result<_, _>>().map_err(|e| db_err("errors.directions.iterate_failed", e))
}

pub fn update_direction(conn: &Connection, id: i64, name: &str) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE directions SET name = ?1 WHERE id = ?2",
        params![name, id],
        &format!("направление id={id} переименовано в \"{name}\""),
        "errors.directions.update_failed",
        "errors.directions.not_found",
    )
}

pub fn archive_direction(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE directions SET isArchived = TRUE WHERE id = ?1",
        params![id],
        &format!("направление id={id} перенесено в архив"),
        "errors.directions.delete_failed",
        "errors.directions.not_found",
    )
}

pub fn restore_direction(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE directions SET isArchived = FALSE WHERE id = ?1",
        params![id],
        &format!("направление id={id} восстановлено из архива"),
        "errors.directions.restore_failed",
        "errors.directions.not_found",
    )
}

// ---------- Факультеты ----------

pub fn create_faculty(conn: &Connection, name: &str) -> AppResult<Faculty> {
    conn.execute("INSERT INTO faculties (name) VALUES (?1)", params![name])
        .map_err(|e| db_err("errors.faculties.create_failed", e))?;
    let id = conn.last_insert_rowid();
    logger::info("faculties", format!("создан факультет id={id} \"{name}\""));
    Ok(Faculty { id, name: name.to_owned(), is_archived: false })
}

pub fn list_faculties(conn: &Connection, archived: bool) -> AppResult<Vec<Faculty>> {
    let mut stmt = conn
        .prepare("SELECT id, name, isArchived FROM faculties WHERE isArchived = ?1 ORDER BY name")
        .map_err(|e| db_err("errors.faculties.get_all_failed", e))?;
    let rows = stmt
        .query_map(params![archived], |row| {
            Ok(Faculty { id: row.get(0)?, name: row.get(1)?, is_archived: row.get(2)? })
        })
        .map_err(|e| db_err("errors.faculties.get_all_failed", e))?;
    rows.collect::<Result<_, _>>().map_err(|e| db_err("errors.faculties.iterate_failed", e))
}

pub fn update_faculty(conn: &Connection, id: i64, name: &str) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE faculties SET name = ?1 WHERE id = ?2",
        params![name, id],
        &format!("факультет id={id} переименован в \"{name}\""),
        "errors.faculties.update_failed",
        "errors.faculties.not_found",
    )
}

pub fn archive_faculty(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE faculties SET isArchived = TRUE WHERE id = ?1",
        params![id],
        &format!("факультет id={id} перенесён в архив"),
        "errors.faculties.delete_failed",
        "errors.faculties.not_found",
    )
}

pub fn restore_faculty(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE faculties SET isArchived = FALSE WHERE id = ?1",
        params![id],
        &format!("факультет id={id} восстановлен из архива"),
        "errors.faculties.update_failed",
        "errors.faculties.not_found",
    )
}

// ---------- Преподаватели ----------

const TEACHER_SELECT: &str = "
    SELECT t.id, t.first_name, t.last_name, COALESCE(t.middle_name, ''), t.direction_id, t.rate,
           COALESCE(d.name, ''), t.isArchived, t.isGuest
    FROM teachers t
    LEFT JOIN directions d ON t.direction_id = d.id";

fn teacher_from_row(row: &Row<'_>) -> rusqlite::Result<Teacher> {
    Ok(Teacher {
        id: row.get(0)?,
        first_name: row.get(1)?,
        last_name: row.get(2)?,
        middle_name: row.get(3)?,
        direction_id: row.get(4)?,
        rate: row.get(5)?,
        direction_name: row.get(6)?,
        is_archived: row.get(7)?,
        is_guest: row.get(8)?,
        is_bound: false,
    })
}

pub fn create_teacher(conn: &Connection, input: &Teacher) -> AppResult<Teacher> {
    conn.execute(
        "INSERT INTO teachers (first_name, last_name, middle_name, direction_id, rate, isGuest)
         VALUES (?1, ?2, ?3, ?4, ?5, FALSE)",
        params![input.first_name, input.last_name, input.middle_name, input.direction_id, input.rate],
    )
    .map_err(|e| db_err("errors.teachers.create_failed", e))?;
    let mut created = input.clone();
    created.id = conn.last_insert_rowid();
    logger::info("teachers", format!("создан преподаватель id={} {}", created.id, created.last_name));
    created.is_archived = false;
    created.is_guest = false;
    Ok(created)
}

pub fn list_teachers(conn: &Connection, archived: bool) -> AppResult<Vec<Teacher>> {
    let sql = format!("{TEACHER_SELECT} WHERE t.isArchived = ?1 ORDER BY t.last_name, t.first_name");
    let mut stmt = conn.prepare(&sql).map_err(|e| db_err("errors.teachers.get_all_failed", e))?;
    let rows = stmt
        .query_map(params![archived], teacher_from_row)
        .map_err(|e| db_err("errors.teachers.get_all_failed", e))?;
    rows.collect::<Result<_, _>>().map_err(|e| db_err("errors.teachers.iterate_failed", e))
}

pub fn update_teacher(conn: &Connection, input: &Teacher) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE teachers SET first_name = ?1, last_name = ?2, middle_name = ?3, direction_id = ?4, rate = ?5 WHERE id = ?6",
        params![input.first_name, input.last_name, input.middle_name, input.direction_id, input.rate, input.id],
        &format!("преподаватель id={} обновлён", input.id),
        "errors.teachers.update_failed",
        "errors.teachers.not_found",
    )
}

pub fn archive_teacher(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE teachers SET isArchived = TRUE WHERE id = ?1",
        params![id],
        &format!("преподаватель id={id} перенесён в архив"),
        "errors.teachers.delete_failed",
        "errors.teachers.not_found",
    )
}

pub fn restore_teacher(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE teachers SET isArchived = FALSE WHERE id = ?1",
        params![id],
        &format!("преподаватель id={id} восстановлен из архива"),
        "errors.teachers.update_failed",
        "errors.teachers.not_found",
    )
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
    let tx = conn.unchecked_transaction().map_err(|e| db_err("errors.semesters.create_failed", e))?;
    tx.execute(
        "INSERT INTO semesters (name, start_date, end_date) VALUES (?1, ?2, ?3)",
        params![name, start, end],
    )
    .map_err(|e| db_err("errors.semesters.create_failed", e))?;
    let id = tx.last_insert_rowid();
    logger::info("semesters", format!("создан семестр id={id} \"{name}\" ({start} — {end})"));
    // Новый семестр получает всех негостевых преподавателей
    tx.execute(
        "INSERT INTO semester_teachers (semester_id, teacher_id)
         SELECT ?1, t.id FROM teachers t WHERE t.isArchived = FALSE AND t.isGuest = FALSE",
        params![id],
    )
    .map_err(|e| db_err("errors.semesters.bind_teachers_failed", e))?;
    tx.commit().map_err(|e| db_err("errors.lessons.commit_failed", e))?;
    Ok(Semester { id, name: name.to_owned(), start_date: start.to_owned(), end_date: end.to_owned(), is_archived: false })
}

pub fn get_semester(conn: &Connection, id: i64) -> AppResult<Semester> {
    conn.query_row(
        "SELECT id, name, start_date, end_date, isArchived FROM semesters WHERE id = ?1",
        params![id],
        semester_from_row,
    )
    .optional()
    .map_err(|e| db_err("errors.semesters.get_failed", e))?
    .ok_or_else(|| not_found("errors.semesters.not_found"))
}

pub fn list_semesters(conn: &Connection, archived: bool) -> AppResult<Vec<Semester>> {
    let mut stmt = conn
        .prepare("SELECT id, name, start_date, end_date, isArchived FROM semesters WHERE isArchived = ?1 ORDER BY start_date DESC")
        .map_err(|e| db_err("errors.semesters.get_all_failed", e))?;
    let rows = stmt
        .query_map(params![archived], semester_from_row)
        .map_err(|e| db_err("errors.semesters.get_all_failed", e))?;
    rows.collect::<Result<_, _>>().map_err(|e| db_err("errors.semesters.iterate_failed", e))
}

pub fn update_semester(conn: &Connection, id: i64, name: &str, start: &str, end: &str) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE semesters SET name = ?1, start_date = ?2, end_date = ?3 WHERE id = ?4",
        params![name, start, end, id],
        &format!("семестр id={id} обновлён: \"{name}\" ({start} — {end})"),
        "errors.semesters.update_failed",
        "errors.semesters.not_found",
    )
}


pub fn archive_semester(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE semesters SET isArchived = TRUE WHERE id = ?1",
        params![id],
        &format!("семестр id={id} перенесён в архив"),
        "errors.semesters.delete_failed",
        "errors.semesters.not_found",
    )
}

pub fn restore_semester(conn: &Connection, id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "UPDATE semesters SET isArchived = FALSE WHERE id = ?1",
        params![id],
        &format!("семестр id={id} восстановлен из архива"),
        "errors.semesters.update_failed",
        "errors.semesters.not_found",
    )
}

/// Все преподаватели, доступные для семестра, с признаком привязки.
pub fn teachers_for_semester(conn: &Connection, semester_id: i64) -> AppResult<Vec<Teacher>> {
    let sql = format!(
        "SELECT t.id, t.first_name, t.last_name, COALESCE(t.middle_name, ''), t.direction_id, t.rate,
                COALESCE(d.name, ''), t.isArchived, t.isGuest,
                CASE WHEN st.teacher_id IS NOT NULL THEN 1 ELSE 0 END
         FROM teachers t
         LEFT JOIN directions d ON t.direction_id = d.id
         LEFT JOIN semester_teachers st ON t.id = st.teacher_id AND st.semester_id = ?1
         ORDER BY t.isArchived, t.last_name, t.first_name"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| db_err("errors.semesters.get_teachers_failed", e))?;
    let rows = stmt
        .query_map(params![semester_id], |row| {
            let mut teacher = teacher_from_row(row)?;
            teacher.is_bound = row.get::<_, i64>(9)? == 1;
            Ok(teacher)
        })
        .map_err(|e| db_err("errors.semesters.get_teachers_failed", e))?;
    rows.collect::<Result<_, _>>().map_err(|e| db_err("errors.teachers.iterate_failed", e))
}

pub fn bind_teacher(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<()> {
    if get_semester(conn, semester_id).is_err() {
        return Err(not_found("errors.semesters.not_found"));
    }

    let teacher_is_guest: bool = conn
        .query_row("SELECT isGuest FROM teachers WHERE id = ?1", params![teacher_id], |row| row.get(0))
        .optional()
        .map_err(|e| db_err("errors.teachers.get_failed", e))?
        .ok_or_else(|| not_found("errors.teachers.not_found"))?;

    let already_bound: bool = conn
        .query_row(
            "SELECT 1 FROM semester_teachers WHERE semester_id = ?1 AND teacher_id = ?2",
            params![semester_id, teacher_id],
            |_| Ok(true),
        )
        .optional()
        .map_err(|e| db_err("errors.semesters.bind_teacher_failed", e))?
        .unwrap_or(false);
    if already_bound {
        return Ok(());
    }

    // Гостевой преподаватель может быть привязан только к одному семестру
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
    Ok(())
}

pub fn unbind_teacher(conn: &Connection, semester_id: i64, teacher_id: i64) -> AppResult<()> {
    exec_checked(
        conn,
        "DELETE FROM semester_teachers WHERE semester_id = ?1 AND teacher_id = ?2",
        params![semester_id, teacher_id],
        "",
        "errors.semesters.unbind_teacher_failed",
        "errors.teachers.not_found",
    )
}

// ---------- Занятия ----------


/// Загружает занятия с именами и связями. Порядок — как в Go-версии.
pub fn lessons_for_semester(conn: &Connection, semester_id: i64) -> AppResult<Vec<Lesson>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, semester_id, day_of_week, start_time, end_time, direction_id, teacher_count
             FROM lessons WHERE semester_id = ?1 ORDER BY day_of_week, start_time",
        )
        .map_err(|e| db_err("errors.lessons.get_all_failed", e))?;
    let base: Vec<Lesson> = stmt
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
        .map_err(|e| db_err("errors.lessons.get_all_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.lessons.iterate_failed", e))?;
    drop(stmt);

    base.into_iter()
        .map(|mut lesson| {
            let (faculty_ids, faculty_names) = lesson_faculties(conn, lesson.id)?;
            let (teacher_ids, teacher_names) = lesson_teachers(conn, lesson.id)?;
            lesson.faculty_ids = faculty_ids;
            lesson.faculty_names = faculty_names;
            lesson.teacher_ids = teacher_ids;
            lesson.teacher_names = teacher_names;
            Ok(lesson)
        })
        .collect()
}

fn lesson_faculties(conn: &Connection, lesson_id: i64) -> AppResult<(Vec<i64>, Vec<String>)> {
    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.name FROM lesson_faculties lf
             JOIN faculties f ON lf.faculty_id = f.id
             WHERE lf.lesson_id = ?1 ORDER BY f.name",
        )
        .map_err(|e| db_err("errors.lessons.get_faculties_failed", e))?;
    let pairs: Vec<(i64, String)> = stmt
        .query_map(params![lesson_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| db_err("errors.lessons.get_faculties_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.lessons.get_faculties_failed", e))?;
    Ok(pairs.into_iter().unzip())
}

fn lesson_teachers(conn: &Connection, lesson_id: i64) -> AppResult<(Vec<i64>, Vec<String>)> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.first_name || ' ' || t.last_name FROM lesson_teachers lt
             JOIN teachers t ON lt.teacher_id = t.id
             WHERE lt.lesson_id = ?1 ORDER BY t.last_name, t.first_name",
        )
        .map_err(|e| db_err("errors.lessons.get_teachers_failed", e))?;
    let pairs: Vec<(i64, String)> = stmt
        .query_map(params![lesson_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| db_err("errors.lessons.get_teachers_failed", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| db_err("errors.lessons.get_teachers_failed", e))?;
    Ok(pairs.into_iter().unzip())
}

fn write_links(conn: &Connection, lesson_id: i64, input: &LessonInput) -> AppResult<()> {
    for faculty_id in &input.faculty_ids {
        conn.execute(
            "INSERT INTO lesson_faculties (lesson_id, faculty_id) VALUES (?1, ?2)",
            params![lesson_id, faculty_id],
        )
        .map_err(|e| db_err("errors.lessons.create_faculty_link_failed", e))?;
    }

    // Лимит преподавателей: нельзя назначить больше, чем указано для пары
    if let Some(limit) = input.teacher_count {
        if input.teacher_ids.len() as i64 > limit {
            return Err(AppError::key_with_arg("errors.lessons.teacher_count_exceeded", limit));
        }
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

pub fn create_lesson(conn: &Connection, input: &LessonInput) -> AppResult<i64> {
    let tx = conn.unchecked_transaction().map_err(|e| db_err("errors.lessons.create_failed", e))?;
    tx.execute(
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
    let id = tx.last_insert_rowid();
    write_links(&tx, id, input)?;
    tx.commit().map_err(|e| db_err("errors.lessons.commit_failed", e))?;
    Ok(id)
}

pub fn update_lesson(conn: &Connection, id: i64, input: &LessonInput) -> AppResult<()> {
    let tx = conn.unchecked_transaction().map_err(|e| db_err("errors.lessons.update_failed", e))?;
    exec_checked(
        &tx,
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
        "",
        "errors.lessons.update_failed",
        "errors.lessons.not_found",
    )?;
    tx.execute("DELETE FROM lesson_faculties WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_faculty_links_failed", e))?;
    tx.execute("DELETE FROM lesson_teachers WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_teacher_links_failed", e))?;
    write_links(&tx, id, input)?;
    tx.commit().map_err(|e| db_err("errors.lessons.commit_failed", e))?;
    Ok(())
}

pub fn delete_lesson(conn: &Connection, id: i64) -> AppResult<()> {
    let tx = conn.unchecked_transaction().map_err(|e| db_err("errors.lessons.delete_failed", e))?;
    tx.execute("DELETE FROM lesson_faculties WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_faculty_links_failed", e))?;
    tx.execute("DELETE FROM lesson_teachers WHERE lesson_id = ?1", params![id])
        .map_err(|e| db_err("errors.lessons.delete_teacher_links_failed", e))?;
    exec_checked(
        &tx,
        "DELETE FROM lessons WHERE id = ?1",
        params![id],
        "",
        "errors.lessons.delete_failed",
        "errors.lessons.not_found",
    )?;
    tx.commit().map_err(|e| db_err("errors.lessons.commit_failed", e))?;
    Ok(())
}
