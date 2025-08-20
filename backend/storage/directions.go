package storage

import (
	"database/sql"
	"fmt"
	"time"
	"tulsu-pe-timetable/backend/locales"
)

// DirectionsRepository репозиторий для работы с направлениями
type DirectionsRepository struct {
	db *sql.DB
}

// NewDirectionsRepository создает новый репозиторий направлений
func NewDirectionsRepository(db *sql.DB) *DirectionsRepository {
	return &DirectionsRepository{db: db}
}

// Create создает новое направление
func (r *DirectionsRepository) Create(req CreateDirectionRequest) (*Direction, error) {
	query := `
		INSERT INTO directions (name, description, created_at, updated_at)
		VALUES (?, ?, ?, ?)
	`
	
	now := time.Now()
	result, err := r.db.Exec(query, req.Name, req.Description, now, now)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.create_failed")+": %w", err)
	}
	
	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.get_id_failed")+": %w", err)
	}
	
	return &Direction{
		ID:          id,
		Name:        req.Name,
		Description: req.Description,
		CreatedAt:   now,
		UpdatedAt:   now,
	}, nil
}

// GetByID получает направление по ID
func (r *DirectionsRepository) GetByID(id int64) (*Direction, error) {
	query := `
		SELECT id, name, description, created_at, updated_at
		FROM directions
		WHERE id = ?
	`
	
	var direction Direction
	err := r.db.QueryRow(query, id).Scan(
		&direction.ID,
		&direction.Name,
		&direction.Description,
		&direction.CreatedAt,
		&direction.UpdatedAt,
	)
	
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf(locales.GetMessage("errors.directions.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.get_failed")+": %w", err)
	}
	
	return &direction, nil
}

// GetAll получает все направления
func (r *DirectionsRepository) GetAll() ([]Direction, error) {
	query := `
		SELECT id, name, description, created_at, updated_at
		FROM directions
		ORDER BY name
	`
	
	rows, err := r.db.Query(query)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.get_all_failed")+": %w", err)
	}
	defer rows.Close()
	
	var directions []Direction
	for rows.Next() {
		var direction Direction
		err := rows.Scan(
			&direction.ID,
			&direction.Name,
			&direction.Description,
			&direction.CreatedAt,
			&direction.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.directions.scan_failed")+": %w", err)
		}
		directions = append(directions, direction)
	}
	
	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.iterate_failed")+": %w", err)
	}
	
	return directions, nil
}

// Update обновляет направление
func (r *DirectionsRepository) Update(req UpdateDirectionRequest) (*Direction, error) {
	query := `
		UPDATE directions
		SET name = ?, description = ?, updated_at = ?
		WHERE id = ?
	`
	
	now := time.Now()
	result, err := r.db.Exec(query, req.Name, req.Description, now, req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.update_failed")+": %w", err)
	}
	
	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.update_check_failed")+": %w", err)
	}
	
	if rowsAffected == 0 {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.not_found"))
	}
	
	return &Direction{
		ID:          req.ID,
		Name:        req.Name,
		Description: req.Description,
		UpdatedAt:   now,
	}, nil
}

// Delete удаляет направление
func (r *DirectionsRepository) Delete(id int64) error {
	query := `DELETE FROM directions WHERE id = ?`
	
	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.directions.delete_failed")+": %w", err)
	}
	
	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.directions.delete_check_failed")+": %w", err)
	}
	
	if rowsAffected == 0 {
		return fmt.Errorf(locales.GetMessage("errors.directions.not_found"))
	}
	
	return nil
}

// Exists проверяет существование направления по ID
func (r *DirectionsRepository) Exists(id int64) (bool, error) {
	query := `SELECT 1 FROM directions WHERE id = ?`
	
	var exists int
	err := r.db.QueryRow(query, id).Scan(&exists)
	if err != nil {
		if err == sql.ErrNoRows {
			return false, nil
		}
		return false, fmt.Errorf(locales.GetMessage("errors.directions.exists_check_failed")+": %w", err)
	}
	
	return true, nil
}
