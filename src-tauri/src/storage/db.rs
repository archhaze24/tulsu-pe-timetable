use std::path::Path;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

const SCHEMA: &str = include_str!("schema.sql");

/// Открывает базу, включает PRAGMA как в Go-версии и применяет схему.
pub fn open(path: &Path) -> AppResult<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| AppError::with_cause("errors.database.create_dir_failed", e))?;
    }

    let conn = Connection::open(path)
        .map_err(|e| AppError::with_cause("errors.database.connection_failed", e))?;

    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;",
    )
    .map_err(|e| AppError::with_cause("errors.database.configure_failed", e))?;

    migrate(&conn).map_err(|e| AppError::with_cause("errors.database.migrations_failed", e))?;

    Ok(conn)
}

/// Схема из `0001_initial.sql` и добавление колонок из `0002_safe_columns.go`.
/// Идемпотентна: на уже существующей базе ничего не меняет.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;

    let safe_columns = [
        ("teachers", "middle_name", "ALTER TABLE teachers ADD COLUMN middle_name TEXT DEFAULT ''"),
        ("teachers", "direction_id", "ALTER TABLE teachers ADD COLUMN direction_id INTEGER NOT NULL DEFAULT 0"),
        ("teachers", "rate", "ALTER TABLE teachers ADD COLUMN rate REAL NOT NULL DEFAULT 1.0"),
        ("lessons", "teacher_count", "ALTER TABLE lessons ADD COLUMN teacher_count INTEGER"),
        ("teachers", "isArchived", "ALTER TABLE teachers ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"),
        ("directions", "isArchived", "ALTER TABLE directions ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"),
        ("faculties", "isArchived", "ALTER TABLE faculties ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"),
        ("semesters", "isArchived", "ALTER TABLE semesters ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"),
    ];

    for (table, column, ddl) in safe_columns {
        if !column_exists(conn, table, column)? {
            conn.execute(ddl, [])?;
        }
    }

    conn.execute("UPDATE teachers SET middle_name = '' WHERE middle_name IS NULL", [])?;
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}
