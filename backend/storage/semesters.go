package storage

import (
	"database/sql"
	"fmt"
	"tulsu-pe-timetable/backend/locales"
)

// SemestersRepository репозиторий для работы с семестрами
type SemestersRepository struct {
	db *sql.DB
}

// NewSemestersRepository создает новый репозиторий семестров
func NewSemestersRepository(db *sql.DB) *SemestersRepository {
	return &SemestersRepository{db: db}
}

// Create создает новый семестр
func (r *SemestersRepository) Create(req CreateSemesterRequest) (*Semester, error) {
	query := `
		INSERT INTO semesters (name, start_date, end_date)
		VALUES (?, ?, ?)
	`

	result, err := r.db.Exec(query, req.Name, req.StartDate, req.EndDate)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.create_failed")+": %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.get_id_failed")+": %w", err)
	}

	return &Semester{
		ID:        id,
		Name:      req.Name,
		StartDate: req.StartDate,
		EndDate:   req.EndDate,
	}, nil
}

// GetByID получает семестр по ID
func (r *SemestersRepository) GetByID(id int64) (*Semester, error) {
	query := `
		SELECT id, name, start_date, end_date
		FROM semesters
		WHERE id = ?
	`

	var semester Semester
	err := r.db.QueryRow(query, id).Scan(
		&semester.ID,
		&semester.Name,
		&semester.StartDate,
		&semester.EndDate,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf(locales.GetMessage("errors.semesters.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.get_failed")+": %w", err)
	}

	return &semester, nil
}

// GetAll получает все семестры
func (r *SemestersRepository) GetAll() ([]Semester, error) {
	query := `
		SELECT id, name, start_date, end_date
		FROM semesters
		ORDER BY start_date DESC
	`

	rows, err := r.db.Query(query)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.get_all_failed")+": %w", err)
	}
	defer rows.Close()

	var semesters []Semester
	for rows.Next() {
		var semester Semester
		err := rows.Scan(
			&semester.ID,
			&semester.Name,
			&semester.StartDate,
			&semester.EndDate,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.semesters.scan_failed")+": %w", err)
		}
		semesters = append(semesters, semester)
	}

	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.iterate_failed")+": %w", err)
	}

	return semesters, nil
}

// Update обновляет семестр
func (r *SemestersRepository) Update(req UpdateSemesterRequest) (*Semester, error) {
	query := `
		UPDATE semesters
		SET name = ?, start_date = ?, end_date = ?
		WHERE id = ?
	`

	result, err := r.db.Exec(query, req.Name, req.StartDate, req.EndDate, req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.update_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.update_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.not_found"))
	}

	return &Semester{
		ID:        req.ID,
		Name:      req.Name,
		StartDate: req.StartDate,
		EndDate:   req.EndDate,
	}, nil
}

// Delete удаляет семестр
func (r *SemestersRepository) Delete(id int64) error {
	query := `DELETE FROM semesters WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.delete_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.delete_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf(locales.GetMessage("errors.semesters.not_found"))
	}

	return nil
}

// Exists проверяет существование семестра по ID
func (r *SemestersRepository) Exists(id int64) (bool, error) {
	query := `SELECT 1 FROM semesters WHERE id = ?`

	var exists int
	err := r.db.QueryRow(query, id).Scan(&exists)
	if err != nil {
		if err == sql.ErrNoRows {
			return false, nil
		}
		return false, fmt.Errorf(locales.GetMessage("errors.semesters.exists_check_failed")+": %w", err)
	}

	return true, nil
}
