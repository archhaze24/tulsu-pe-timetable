package storage

import (
	"database/sql"
	"fmt"
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
		INSERT INTO directions (name)
		VALUES (?)
	`

	result, err := r.db.Exec(query, req.Name)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.create_failed")+": %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.get_id_failed")+": %w", err)
	}

	return &Direction{
		ID:         id,
		Name:       req.Name,
		IsArchived: false,
	}, nil
}

// GetByID получает направление по ID
func (r *DirectionsRepository) GetByID(id int64) (*Direction, error) {
	query := `
		SELECT id, name, isArchived
		FROM directions
		WHERE id = ?
	`

	var direction Direction
	err := r.db.QueryRow(query, id).Scan(
		&direction.ID,
		&direction.Name,
		&direction.IsArchived,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf("%s", locales.GetMessage("errors.directions.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.get_failed")+": %w", err)
	}

	return &direction, nil
}

// GetAll получает все направления
func (r *DirectionsRepository) GetAll() ([]Direction, error) {
	return r.GetAllByArchived(false)
}

// GetAllByArchived получает направления по признаку архива
func (r *DirectionsRepository) GetAllByArchived(isArchived bool) ([]Direction, error) {
	query := `
		SELECT id, name, isArchived
		FROM directions
		WHERE isArchived = ?
		ORDER BY name
	`

	rows, err := r.db.Query(query, isArchived)
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
			&direction.IsArchived,
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
		SET name = ?
		WHERE id = ?
	`

	result, err := r.db.Exec(query, req.Name, req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.update_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.directions.update_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return nil, fmt.Errorf("%s", locales.GetMessage("errors.directions.not_found"))
	}

	return &Direction{
		ID:         req.ID,
		Name:       req.Name,
		IsArchived: false,
	}, nil
}

// Delete мягко удаляет направление (архивирует)
func (r *DirectionsRepository) Delete(id int64) error {
	query := `UPDATE directions SET isArchived = TRUE WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.directions.delete_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.directions.delete_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf("%s", locales.GetMessage("errors.directions.not_found"))
	}

	return nil
}

// Restore восстанавливает направление из архива
func (r *DirectionsRepository) Restore(id int64) error {
	query := `UPDATE directions SET isArchived = FALSE WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.directions.update_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.directions.update_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf("%s", locales.GetMessage("errors.directions.not_found"))
	}

	return nil
}
