package storage

import (
	"database/sql"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"tulsu-pe-timetable/backend/config"
	"tulsu-pe-timetable/backend/locales"

	_ "modernc.org/sqlite"
)

// Database представляет соединение с базой данных
type Database struct {
	db *sql.DB
}

// NewDatabase создает новое соединение с базой данных
func NewDatabase(cfg *config.Config) (*Database, error) {
	// Создаем директорию для базы данных, если её нет
	dbDir := filepath.Dir(cfg.DbPath)
	if err := os.MkdirAll(dbDir, 0755); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.database.create_dir_failed")+": %w", err)
	}

	// Открываем соединение с базой данных
	db, err := sql.Open("sqlite", cfg.DbPath)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.database.connection_failed")+": %w", err)
	}

	// Проверяем соединение
	if err := db.Ping(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.database.ping_failed")+": %w", err)
	}

	database := &Database{db: db}

	// Инициализируем таблицы
	if err := database.initTables(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.database.init_tables_failed")+": %w", err)
	}

	log.Printf("База данных успешно инициализирована: %s", cfg.DbPath)
	return database, nil
}

// Close закрывает соединение с базой данных
func (d *Database) Close() error {
	if d.db != nil {
		return d.db.Close()
	}
	return nil
}

// GetDB возвращает указатель на sql.DB для прямого доступа
func (d *Database) GetDB() *sql.DB {
	return d.db
}

// initTables создает все необходимые таблицы в базе данных
func (d *Database) initTables() error {
	tables := []string{
		createDirectionsTable,
		createFacultiesTable,
		createTeachersTable,
		createSemestersTable,
		createLessonsTable,
		createLessonFacultiesTable,
		createLessonTeachersTable,
		createSemesterTeachersTable,
	}

	for _, tableSQL := range tables {
		if _, err := d.db.Exec(tableSQL); err != nil {
			return fmt.Errorf("ошибка создания таблицы: %w", err)
		}
	}

	// Выполняем миграции схемы для существующих баз данных
	if err := d.migrateSchema(); err != nil {
		return err
	}

	return nil
}

// SQL для создания таблиц
const (
	createDirectionsTable = `
	CREATE TABLE IF NOT EXISTS directions (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL
	);`

	createFacultiesTable = `
	CREATE TABLE IF NOT EXISTS faculties (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL
	);`

	createTeachersTable = `
	CREATE TABLE IF NOT EXISTS teachers (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		first_name TEXT NOT NULL,
		last_name TEXT NOT NULL,
		isArchived BOOLEAN NOT NULL DEFAULT FALSE,
		isGuest BOOLEAN NOT NULL DEFAULT FALSE,
		middle_name TEXT,
		direction_id INTEGER NOT NULL,
		rate REAL NOT NULL,
		FOREIGN KEY (direction_id) REFERENCES directions (id)
	);`

	createSemestersTable = `
	CREATE TABLE IF NOT EXISTS semesters (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL,
		start_date DATE NOT NULL,
		end_date DATE NOT NULL
	);`

	createLessonsTable = `
	CREATE TABLE IF NOT EXISTS lessons (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		semester_id INTEGER NOT NULL,
		day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 1 AND 7),
		start_time TIME NOT NULL,
		end_time TIME NOT NULL,
		direction_id INTEGER NOT NULL,
		teacher_count INTEGER,
		FOREIGN KEY (semester_id) REFERENCES semesters (id),
		FOREIGN KEY (direction_id) REFERENCES directions (id)
	);`

	createLessonFacultiesTable = `
	CREATE TABLE IF NOT EXISTS lesson_faculties (
		lesson_id INTEGER NOT NULL,
		faculty_id INTEGER NOT NULL,
		PRIMARY KEY (lesson_id, faculty_id),
		FOREIGN KEY (lesson_id) REFERENCES lessons (id),
		FOREIGN KEY (faculty_id) REFERENCES faculties (id)
	);`

	createLessonTeachersTable = `
	CREATE TABLE IF NOT EXISTS lesson_teachers (
		lesson_id INTEGER NOT NULL,
		teacher_id INTEGER NOT NULL,
		PRIMARY KEY (lesson_id, teacher_id),
		FOREIGN KEY (lesson_id) REFERENCES lessons (id),
		FOREIGN KEY (teacher_id) REFERENCES teachers (id)
	);`

	createSemesterTeachersTable = `
	CREATE TABLE IF NOT EXISTS semester_teachers (
		semester_id INTEGER NOT NULL,
		teacher_id INTEGER NOT NULL,
		PRIMARY KEY (semester_id, teacher_id),
		FOREIGN KEY (semester_id) REFERENCES semesters (id),
		FOREIGN KEY (teacher_id) REFERENCES teachers (id)
	);`
)

// columnExists проверяет наличие столбца в таблице (SQLite)
func columnExists(db *sql.DB, tableName string, columnName string) (bool, error) {
	// PRAGMA table_info returns: cid, name, type, notnull, dflt_value, pk
	rows, err := db.Query("PRAGMA table_info(" + tableName + ")")
	if err != nil {
		return false, err
	}
	defer rows.Close()

	for rows.Next() {
		var cid int
		var name, colType string
		var notnull, pk int
		var dflt sql.NullString
		if err := rows.Scan(&cid, &name, &colType, &notnull, &dflt, &pk); err != nil {
			return false, err
		}
		if name == columnName {
			return true, nil
		}
	}
	if err := rows.Err(); err != nil {
		return false, err
	}
	return false, nil
}

// migrateSchema выполняет безопасные миграции для существующих баз данных,
// добавляя недостающие столбцы с корректными значениями по умолчанию
func (d *Database) migrateSchema() error {
	type addColumnPlan struct {
		table  string
		column string
		ddl    string
	}

	plans := []addColumnPlan{
		// directions.address используется в CRUD запросах
		{table: "directions", column: "address", ddl: "ALTER TABLE directions ADD COLUMN address TEXT DEFAULT ''"},

		// teachers.middle_name сканируется в string
		{table: "teachers", column: "middle_name", ddl: "ALTER TABLE teachers ADD COLUMN middle_name TEXT DEFAULT ''"},

		// teachers.direction_id участвует в JOIN
		{table: "teachers", column: "direction_id", ddl: "ALTER TABLE teachers ADD COLUMN direction_id INTEGER NOT NULL DEFAULT 0"},

		// teachers.rate хранит ставку преподавателя
		{table: "teachers", column: "rate", ddl: "ALTER TABLE teachers ADD COLUMN rate REAL NOT NULL DEFAULT 1.0"},

		// lessons.teacher_count количество преподавателей на занятии
		{table: "lessons", column: "teacher_count", ddl: "ALTER TABLE lessons ADD COLUMN teacher_count INTEGER"},
	}

	for _, plan := range plans {
		exists, err := columnExists(d.db, plan.table, plan.column)
		if err != nil {
			return fmt.Errorf("не удалось проверить столбец %s.%s: %w", plan.table, plan.column, err)
		}
		if !exists {
			if _, err := d.db.Exec(plan.ddl); err != nil {
				return fmt.Errorf("ошибка миграции: добавление столбца %s.%s: %w", plan.table, plan.column, err)
			}
		}
	}

	// Нормализуем NULL → '' для текстовых столбцов, которые мы сканируем в string
	// directions.address
	if exists, err := columnExists(d.db, "directions", "address"); err != nil {
		return fmt.Errorf("не удалось проверить столбец directions.address: %w", err)
	} else if exists {
		if _, err := d.db.Exec("UPDATE directions SET address = '' WHERE address IS NULL"); err != nil {
			return fmt.Errorf("не удалось нормализовать NULL в directions.address: %w", err)
		}
	}

	// teachers.middle_name
	if exists, err := columnExists(d.db, "teachers", "middle_name"); err != nil {
		return fmt.Errorf("не удалось проверить столбец teachers.middle_name: %w", err)
	} else if exists {
		if _, err := d.db.Exec("UPDATE teachers SET middle_name = '' WHERE middle_name IS NULL"); err != nil {
			return fmt.Errorf("не удалось нормализовать NULL в teachers.middle_name: %w", err)
		}
	}

	// faculties.short_name удалён из схемы; никаких действий не требуется

	return nil
}
