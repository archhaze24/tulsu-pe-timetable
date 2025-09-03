package storage

import (
	"database/sql"
	"fmt"
	"tulsu-pe-timetable/backend/locales"
)

// FacultiesRepository репозиторий для работы с факультетами
type FacultiesRepository struct {
	db *sql.DB
}

// NewFacultiesRepository создает новый репозиторий факультетов
func NewFacultiesRepository(db *sql.DB) *FacultiesRepository {
	return &FacultiesRepository{db: db}
}

// Create создает новый факультет
func (r *FacultiesRepository) Create(req CreateFacultyRequest) (*Faculty, error) {
	query := `
		INSERT INTO faculties (name)
		VALUES (?)
	`

	result, err := r.db.Exec(query, req.Name)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.create_failed")+": %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.get_id_failed")+": %w", err)
	}

	return &Faculty{
		ID:   id,
		Name: req.Name,
	}, nil
}

// GetByID получает факультет по ID
func (r *FacultiesRepository) GetByID(id int64) (*Faculty, error) {
	query := `
		SELECT id, name
		FROM faculties
		WHERE id = ?
	`

	var faculty Faculty
	err := r.db.QueryRow(query, id).Scan(
		&faculty.ID,
		&faculty.Name,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf(locales.GetMessage("errors.faculties.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.get_failed")+": %w", err)
	}

	return &faculty, nil
}

// GetAll получает все факультеты
func (r *FacultiesRepository) GetAll() ([]Faculty, error) {
	query := `
		SELECT id, name
		FROM faculties
		ORDER BY name
	`

	rows, err := r.db.Query(query)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.get_all_failed")+": %w", err)
	}
	defer rows.Close()

	var faculties []Faculty
	for rows.Next() {
		var faculty Faculty
		err := rows.Scan(
			&faculty.ID,
			&faculty.Name,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.faculties.scan_failed")+": %w", err)
		}
		faculties = append(faculties, faculty)
	}

	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.iterate_failed")+": %w", err)
	}

	return faculties, nil
}

// Update обновляет факультет
func (r *FacultiesRepository) Update(req UpdateFacultyRequest) (*Faculty, error) {
	query := `
		UPDATE faculties
		SET name = ?
		WHERE id = ?
	`

	result, err := r.db.Exec(query, req.Name, req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.update_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.update_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return nil, fmt.Errorf(locales.GetMessage("errors.faculties.not_found"))
	}

	return &Faculty{
		ID:   req.ID,
		Name: req.Name,
	}, nil
}

// Delete удаляет факультет
func (r *FacultiesRepository) Delete(id int64) error {
	query := `DELETE FROM faculties WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.faculties.delete_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.faculties.delete_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf(locales.GetMessage("errors.faculties.not_found"))
	}

	return nil
}

// Exists проверяет существование факультета по ID
func (r *FacultiesRepository) Exists(id int64) (bool, error) {
	query := `SELECT 1 FROM faculties WHERE id = ?`

	var exists int
	err := r.db.QueryRow(query, id).Scan(&exists)
	if err != nil {
		if err == sql.ErrNoRows {
			return false, nil
		}
		return false, fmt.Errorf(locales.GetMessage("errors.faculties.exists_check_failed")+": %w", err)
	}

	return true, nil
}
