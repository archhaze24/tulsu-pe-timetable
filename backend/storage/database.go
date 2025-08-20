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
		createLessonsTable,
	}

	for _, tableSQL := range tables {
		if _, err := d.db.Exec(tableSQL); err != nil {
			return fmt.Errorf("ошибка создания таблицы: %w", err)
		}
	}

	return nil
}

// SQL для создания таблиц
const (
	createDirectionsTable = `
	CREATE TABLE IF NOT EXISTS directions (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL UNIQUE,
		description TEXT,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);`

	createFacultiesTable = `
	CREATE TABLE IF NOT EXISTS faculties (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL UNIQUE,
		short_name TEXT NOT NULL UNIQUE,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);`

	createTeachersTable = `
	CREATE TABLE IF NOT EXISTS teachers (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		last_name TEXT NOT NULL,
		first_name TEXT NOT NULL,
		middle_name TEXT,
		rate REAL NOT NULL DEFAULT 1.0,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);`

	createLessonsTable = `
	CREATE TABLE IF NOT EXISTS lessons (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		faculty_id INTEGER NOT NULL,
		direction_id INTEGER NOT NULL,
		teacher_id INTEGER,
		day_of_week INTEGER NOT NULL CHECK (day_of_week >= 1 AND day_of_week <= 7),
		lesson_number INTEGER NOT NULL CHECK (lesson_number >= 1 AND lesson_number <= 8),
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (faculty_id) REFERENCES faculties (id) ON DELETE CASCADE,
		FOREIGN KEY (direction_id) REFERENCES directions (id) ON DELETE CASCADE,
		FOREIGN KEY (teacher_id) REFERENCES teachers (id) ON DELETE SET NULL,
		UNIQUE (faculty_id, day_of_week, lesson_number)
	);`
)
