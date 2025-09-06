package migrations

import (
	"database/sql"
	"fmt"

	goose "github.com/pressly/goose/v3"
)

func init() {
	goose.AddMigration(upSafeColumns, downSafeColumns)
}

// upSafeColumns adds missing columns safely for legacy DBs
func upSafeColumns(tx *sql.Tx) error {
	type addColumnPlan struct {
		table  string
		column string
		ddl    string
	}

	plans := []addColumnPlan{
		{table: "teachers", column: "middle_name", ddl: "ALTER TABLE teachers ADD COLUMN middle_name TEXT DEFAULT ''"},
		{table: "teachers", column: "direction_id", ddl: "ALTER TABLE teachers ADD COLUMN direction_id INTEGER NOT NULL DEFAULT 0"},
		{table: "teachers", column: "rate", ddl: "ALTER TABLE teachers ADD COLUMN rate REAL NOT NULL DEFAULT 1.0"},
		{table: "lessons", column: "teacher_count", ddl: "ALTER TABLE lessons ADD COLUMN teacher_count INTEGER"},
		{table: "teachers", column: "isArchived", ddl: "ALTER TABLE teachers ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"},
		{table: "directions", column: "isArchived", ddl: "ALTER TABLE directions ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"},
		{table: "faculties", column: "isArchived", ddl: "ALTER TABLE faculties ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"},
		{table: "semesters", column: "isArchived", ddl: "ALTER TABLE semesters ADD COLUMN isArchived BOOLEAN NOT NULL DEFAULT FALSE"},
	}

	for _, plan := range plans {
		exists, err := columnExists(tx, plan.table, plan.column)
		if err != nil {
			return fmt.Errorf("column check failed %s.%s: %w", plan.table, plan.column, err)
		}
		if !exists {
			if _, err := tx.Exec(plan.ddl); err != nil {
				return fmt.Errorf("add column failed %s.%s: %w", plan.table, plan.column, err)
			}
		}
	}

	// Normalize NULLs to '' for middle_name
	if exists, err := columnExists(tx, "teachers", "middle_name"); err != nil {
		return fmt.Errorf("column check failed teachers.middle_name: %w", err)
	} else if exists {
		if _, err := tx.Exec("UPDATE teachers SET middle_name = '' WHERE middle_name IS NULL"); err != nil {
			return fmt.Errorf("normalize NULL failed for teachers.middle_name: %w", err)
		}
	}

	return nil
}

// downSafeColumns performs best-effort down migration (no-op due to SQLite limitations)
func downSafeColumns(tx *sql.Tx) error {
	// No-op: dropping columns in SQLite is non-trivial and destructive
	return nil
}

// columnExists checks presence of a column in SQLite table using PRAGMA table_info
func columnExists(qe interface {
	Query(string, ...any) (*sql.Rows, error)
}, tableName string, columnName string) (bool, error) {
	rows, err := qe.Query("PRAGMA table_info(" + tableName + ")")
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
