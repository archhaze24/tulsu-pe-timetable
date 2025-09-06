package storage

import (
	"database/sql"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"tulsu-pe-timetable/backend/config"
	"tulsu-pe-timetable/backend/locales"
	"tulsu-pe-timetable/backend/migrations"

	goose "github.com/pressly/goose/v3"
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

	// Применяем миграции
	if err := database.runMigrations(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.database.migrations_failed")+": %w", err)
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

// runMigrations запускает миграции goose из встроенного FS
func (d *Database) runMigrations() error {
	goose.SetBaseFS(migrations.FS)
	if err := goose.SetDialect("sqlite3"); err != nil {
		return err
	}
	// Встроенный путь миграций — корень FS
	if err := goose.Up(d.db, "."); err != nil {
		return err
	}
	return nil
}
