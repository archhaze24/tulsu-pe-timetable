use std::collections::HashSet;
use std::path::Path;

use chrono::Local;
use rusqlite::Connection;

use crate::error::{AppError, AppResult};
use crate::{archive, logger, schedule};

const SCHEMA: &str = include_str!("schema.sql");

/// Миграции данных и схемы. Каждая выполняется один раз (таблица `app_migrations`) и в транзакции.
type Migration = (&'static str, fn(&Connection) -> AppResult<()>);

const MIGRATIONS: [Migration; 5] = [
    ("teachers_direction_without_fk", teachers_direction_without_fk),
    ("normalize_lesson_times", normalize_lesson_times),
    ("empty_middle_names", empty_middle_names),
    ("remove_stale_assignments", remove_stale_assignments),
    ("teachers_direction_nullable", teachers_direction_nullable),
];

/// Открывает базу, настраивает соединение и применяет схему и миграции.
pub fn open(path: &Path) -> AppResult<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| AppError::with_cause("errors.database.create_dir_failed", e))?;
    }
    let is_new = !path.exists();
    let conn = Connection::open(path).map_err(|e| AppError::with_cause("errors.database.connection_failed", e))?;
    init(&conn)?;
    if is_new {
        logger::warn("db", format!("создана новая пустая база: {}", path.display()));
    }
    Ok(conn)
}

/// Схема из `schema.sql` (как в Go-версии) и добавление колонок старых баз.
/// Идемпотентна: на уже подготовленной базе ничего не меняет.
pub fn init(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;",
    )
    .map_err(|e| AppError::with_cause("errors.database.configure_failed", e))?;

    let fresh = table_count(conn).map_err(migrations_failed)? == 0;
    conn.execute_batch(SCHEMA).map_err(migrations_failed)?;
    add_safe_columns(conn).map_err(migrations_failed)?;
    run_migrations(conn, fresh)?;
    // Индексы создаём после миграций: пересборка таблицы `teachers` удаляет её индексы.
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_lessons_semester ON lessons (semester_id);
         CREATE INDEX IF NOT EXISTS idx_lesson_teachers_teacher ON lesson_teachers (teacher_id);
         CREATE INDEX IF NOT EXISTS idx_semester_teachers_teacher ON semester_teachers (teacher_id);
         CREATE INDEX IF NOT EXISTS idx_teachers_direction ON teachers (direction_id);",
    )
    .map_err(migrations_failed)
}

fn migrations_failed(e: rusqlite::Error) -> AppError {
    AppError::with_cause("errors.database.migrations_failed", e)
}

fn table_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'", [], |row| row.get(0))
}

fn add_safe_columns(conn: &Connection) -> rusqlite::Result<()> {
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

fn run_migrations(conn: &Connection, fresh: bool) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_migrations (
             name TEXT PRIMARY KEY,
             applied_at TEXT NOT NULL DEFAULT (datetime('now'))
         );",
    )
    .map_err(migrations_failed)?;

    let applied: HashSet<String> = {
        let mut stmt = conn.prepare("SELECT name FROM app_migrations").map_err(migrations_failed)?;
        stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(migrations_failed)?
            .collect::<Result<_, _>>()
            .map_err(migrations_failed)?
    };
    let pending: Vec<_> = MIGRATIONS.iter().filter(|(name, _)| !applied.contains(*name)).collect();
    if pending.is_empty() {
        return Ok(());
    }

    if !fresh {
        backup_before_migration(conn)?;
    }

    // Пересборка таблиц требует выключенных внешних ключей. Включаем обратно при любом исходе.
    conn.execute_batch("PRAGMA foreign_keys = OFF;").map_err(migrations_failed)?;
    let result = apply_pending(conn, &pending);
    conn.execute_batch("PRAGMA foreign_keys = ON;").map_err(migrations_failed)?;
    result
}

fn apply_pending(conn: &Connection, pending: &[&Migration]) -> AppResult<()> {
    for (name, run) in pending {
        let tx = conn.unchecked_transaction().map_err(migrations_failed)?;
        run(&tx).map_err(|e| {
            logger::error("db", format!("миграция {name} не применена: {e}"));
            e
        })?;
        tx.execute("INSERT INTO app_migrations (name) VALUES (?1)", [name]).map_err(migrations_failed)?;
        tx.commit().map_err(migrations_failed)?;
        logger::info("db", format!("миграция применена: {name}"));
    }
    Ok(())
}

/// Копия базы через `VACUUM INTO`: в режиме WAL копия одного файла может потерять данные из `-wal`.
fn backup_before_migration(conn: &Connection) -> AppResult<()> {
    let file: String = conn
        .query_row("SELECT file FROM pragma_database_list WHERE name = 'main'", [], |row| row.get(0))
        .map_err(|e| AppError::with_cause("errors.database.backup_failed", e))?;
    if file.is_empty() {
        return Ok(());
    }
    let source = Path::new(&file);
    let stamp = Local::now().format("%Y%m%d-%H%M%S-%3f");
    let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("timetable");
    let target = source.with_file_name(format!("{stem}-backup-{stamp}.db"));
    let target_str = target.to_string_lossy().into_owned();
    conn.execute("VACUUM INTO ?1", [&target_str])
        .map_err(|e| AppError::with_cause("errors.database.backup_failed", e))?;
    match archive::zip_file(&target) {
        Ok(zipped) => logger::info("db", format!("резервная копия перед миграцией: {}", zipped.display())),
        Err(e) => logger::warn("db", format!("резервная копия осталась без сжатия (zip не создан): {e}")),
    }
    Ok(())
}

fn teachers_direction_without_fk(conn: &Connection) -> AppResult<()> {
    let has_fk = {
        let mut stmt = conn
            .prepare("PRAGMA foreign_key_list(teachers)")
            .map_err(migrations_failed)?;
        let mut rows = stmt.query([]).map_err(migrations_failed)?;
        let mut found = false;
        while let Some(row) = rows.next().map_err(migrations_failed)? {
            let table: String = row.get(2).map_err(migrations_failed)?;
            let from: String = row.get(3).map_err(migrations_failed)?;
            found |= table == "directions" && from == "direction_id";
        }
        found
    };
    if !has_fk {
        return Ok(());
    }

    // Направление 0 означает «без направления», поэтому внешний ключ на directions не нужен.
    // Пересборка по рекомендованной схеме SQLite: новая таблица, копия, замена.
    conn.execute_batch(
        "CREATE TABLE teachers_new (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             first_name TEXT NOT NULL,
             last_name TEXT NOT NULL,
             isArchived BOOLEAN NOT NULL DEFAULT FALSE,
             isGuest BOOLEAN NOT NULL DEFAULT FALSE,
             middle_name TEXT,
             direction_id INTEGER NOT NULL DEFAULT 0,
             rate REAL NOT NULL
         );
         INSERT INTO teachers_new (id, first_name, last_name, isArchived, isGuest, middle_name, direction_id, rate)
             SELECT id, first_name, last_name, isArchived, isGuest, middle_name, direction_id, rate FROM teachers;
         UPDATE sqlite_sequence SET seq = (SELECT MAX(seq) FROM sqlite_sequence WHERE name IN ('teachers', 'teachers_new'))
             WHERE name = 'teachers_new';
         DROP TABLE teachers;
         ALTER TABLE teachers_new RENAME TO teachers;",
    )
    .map_err(migrations_failed)
}

/// «Без направления» хранится как NULL, а не как 0: внешний ключ снова действует, а направление 0 не нужно.
/// Ссылки на несуществующие направления тоже становятся NULL, иначе внешний ключ не даст их сохранить.
fn teachers_direction_nullable(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE teachers_new (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             first_name TEXT NOT NULL,
             last_name TEXT NOT NULL,
             isArchived BOOLEAN NOT NULL DEFAULT FALSE,
             isGuest BOOLEAN NOT NULL DEFAULT FALSE,
             middle_name TEXT,
             direction_id INTEGER,
             rate REAL NOT NULL,
             FOREIGN KEY (direction_id) REFERENCES directions (id)
         );
         INSERT INTO teachers_new (id, first_name, last_name, isArchived, isGuest, middle_name, direction_id, rate)
             SELECT id, first_name, last_name, isArchived, isGuest, middle_name,
                    CASE WHEN direction_id IN (SELECT id FROM directions) THEN direction_id END,
                    rate
             FROM teachers;
         UPDATE sqlite_sequence SET seq = (SELECT MAX(seq) FROM sqlite_sequence WHERE name IN ('teachers', 'teachers_new'))
             WHERE name = 'teachers_new';
         DROP TABLE teachers;
         ALTER TABLE teachers_new RENAME TO teachers;",
    )
    .map_err(migrations_failed)
}

/// Время без ведущего нуля («9:00») превращаем в «09:00», иначе ячейки расписания не находят пары.
fn normalize_lesson_times(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "UPDATE lessons SET start_time = '0' || start_time WHERE length(start_time) = 4 AND substr(start_time, 2, 1) = ':';
         UPDATE lessons SET end_time = '0' || end_time WHERE length(end_time) = 4 AND substr(end_time, 2, 1) = ':';",
    )
    .map_err(migrations_failed)
}

fn empty_middle_names(conn: &Connection) -> AppResult<()> {
    conn.execute("UPDATE teachers SET middle_name = '' WHERE middle_name IS NULL", [])
        .map(|_| ())
        .map_err(migrations_failed)
}

/// Назначения преподавателей, которые не привязаны к семестру, раньше оставались невидимыми и занимали лимит.
/// Снимаем их теми же правилами, что и при снятии привязки.
fn remove_stale_assignments(conn: &Connection) -> AppResult<()> {
    let stale: Vec<(i64, i64)> = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT l.semester_id, lt.teacher_id
                 FROM lesson_teachers lt
                 JOIN lessons l ON l.id = lt.lesson_id
                 LEFT JOIN semester_teachers st ON st.semester_id = l.semester_id AND st.teacher_id = lt.teacher_id
                 WHERE st.teacher_id IS NULL",
            )
            .map_err(migrations_failed)?;
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(migrations_failed)?
            .collect::<Result<_, _>>()
            .map_err(migrations_failed)?
    };

    for (semester_id, teacher_id) in stale {
        let removed = schedule::remove_assignments(conn, semester_id, teacher_id)?;
        logger::warn(
            "db",
            format!(
                "снято назначений непривязанного преподавателя id={teacher_id} в семестре id={semester_id}: назначений {}, пар удалено {}",
                removed.assignments, removed.deleted_pairs
            ),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_database_is_migrated_once() {
        let conn = Connection::open_in_memory().unwrap();
        // Старая база: внешний ключ на направление, время без ведущего нуля, назначение непривязанного преподавателя.
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn.execute_batch(
            "INSERT INTO directions (id, name) VALUES (1, 'Футбол');
             INSERT INTO faculties (id, name) VALUES (1, 'Экономики');
             INSERT INTO teachers (id, first_name, last_name, middle_name, direction_id, rate) VALUES
                 (1, 'Иван', 'Иванов', NULL, 1, 1.0),
                 (2, 'Пётр', 'Петров', '', 0, 0.5),
                 (3, 'Анна', 'Сидорова', NULL, 99, 1.0);
             INSERT INTO semesters (id, name, start_date, end_date) VALUES (1, 'Осень', '2026-09-01', '2026-12-31');
             INSERT INTO semester_teachers (semester_id, teacher_id) VALUES (1, 1);
             INSERT INTO lessons (id, semester_id, day_of_week, start_time, end_time, direction_id, teacher_count)
                 VALUES (1, 1, 1, '9:00', '10:35', 1, 2);
             INSERT INTO lesson_faculties (lesson_id, faculty_id) VALUES (1, 1);
             INSERT INTO lesson_teachers (lesson_id, teacher_id) VALUES (1, 1), (1, 2);",
        )
        .unwrap();

        init(&conn).unwrap();

        let fk_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_foreign_key_list('teachers') WHERE \"table\" = 'directions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(fk_count, 1);

        let start: String = conn.query_row("SELECT start_time FROM lessons WHERE id = 1", [], |row| row.get(0)).unwrap();
        assert_eq!(start, "09:00");

        let stale: i64 = conn
            .query_row("SELECT COUNT(*) FROM lesson_teachers WHERE teacher_id = 2", [], |row| row.get(0))
            .unwrap();
        assert_eq!(stale, 0);

        let kept: i64 = conn
            .query_row("SELECT COUNT(*) FROM lesson_teachers WHERE teacher_id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(kept, 1);

        // «Без направления» и ссылка на несуществующее направление становятся NULL.
        let without_direction: i64 = conn
            .query_row("SELECT COUNT(*) FROM teachers WHERE direction_id IS NULL", [], |row| row.get(0))
            .unwrap();
        assert_eq!(without_direction, 2);

        // Повторный запуск ничего не меняет и не падает.
        init(&conn).unwrap();
        let applied: i64 = conn.query_row("SELECT COUNT(*) FROM app_migrations", [], |row| row.get(0)).unwrap();
        assert_eq!(applied, MIGRATIONS.len() as i64);
    }
}
