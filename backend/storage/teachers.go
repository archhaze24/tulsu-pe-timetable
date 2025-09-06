package storage

import (
	"database/sql"
	"errors"
	"fmt"
	"tulsu-pe-timetable/backend/locales"
)

// TeachersRepository репозиторий для работы с преподавателями
type TeachersRepository struct {
	db *sql.DB
}

// NewTeachersRepository создает новый репозиторий преподавателей
func NewTeachersRepository(db *sql.DB) *TeachersRepository {
	return &TeachersRepository{db: db}
}

// Create создает нового преподавателя
func (r *TeachersRepository) Create(req CreateTeacherRequest) (*Teacher, error) {
	query := `
		INSERT INTO teachers (first_name, last_name, middle_name, direction_id, rate)
		VALUES (?, ?, ?, ?, ?)
	`

	result, err := r.db.Exec(query, req.FirstName, req.LastName, req.MiddleName, req.DirectionID, req.Rate)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.create_failed")+": %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.get_id_failed")+": %w", err)
	}

	return &Teacher{
		ID:          id,
		FirstName:   req.FirstName,
		LastName:    req.LastName,
		MiddleName:  req.MiddleName,
		DirectionID: req.DirectionID,
		Rate:        req.Rate,
		IsArchived:  false,
	}, nil
}

// GetByID получает преподавателя по ID
func (r *TeachersRepository) GetByID(id int64) (*Teacher, error) {
	query := `
		SELECT t.id, t.first_name, t.last_name, t.middle_name, t.direction_id, t.rate, d.name, t.isArchived
		FROM teachers t
		LEFT JOIN directions d ON t.direction_id = d.id
		WHERE t.id = ?
	`

	var teacher Teacher
	var directionName sql.NullString
	err := r.db.QueryRow(query, id).Scan(
		&teacher.ID,
		&teacher.FirstName,
		&teacher.LastName,
		&teacher.MiddleName,
		&teacher.DirectionID,
		&teacher.Rate,
		&directionName,
		&teacher.IsArchived,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, errors.New(locales.GetMessage("errors.teachers.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.get_failed")+": %w", err)
	}

	if directionName.Valid {
		teacher.DirectionName = directionName.String
	}

	return &teacher, nil
}

// GetAll получает всех неархивных преподавателей
func (r *TeachersRepository) GetAll() ([]Teacher, error) {
	return r.GetAllByArchived(false)
}

// GetAllByArchived получает всех преподавателей по признаку архивации
func (r *TeachersRepository) GetAllByArchived(isArchived bool) ([]Teacher, error) {
	query := `
		SELECT t.id, t.first_name, t.last_name, t.middle_name, t.direction_id, t.rate, d.name, t.isArchived
		FROM teachers t
		LEFT JOIN directions d ON t.direction_id = d.id
		WHERE t.isArchived = ?
		ORDER BY t.last_name, t.first_name
	`

	rows, err := r.db.Query(query, isArchived)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.get_all_failed")+": %w", err)
	}
	defer rows.Close()

	var teachers []Teacher
	for rows.Next() {
		var teacher Teacher
		var directionName sql.NullString
		err := rows.Scan(
			&teacher.ID,
			&teacher.FirstName,
			&teacher.LastName,
			&teacher.MiddleName,
			&teacher.DirectionID,
			&teacher.Rate,
			&directionName,
			&teacher.IsArchived,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.teachers.scan_failed")+": %w", err)
		}
		if directionName.Valid {
			teacher.DirectionName = directionName.String
		}
		teachers = append(teachers, teacher)
	}

	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.iterate_failed")+": %w", err)
	}

	return teachers, nil
}

// Update обновляет преподавателя
func (r *TeachersRepository) Update(req UpdateTeacherRequest) (*Teacher, error) {
	query := `
		UPDATE teachers
		SET first_name = ?, last_name = ?, middle_name = ?, direction_id = ?, rate = ?
		WHERE id = ?
	`

	result, err := r.db.Exec(query, req.FirstName, req.LastName, req.MiddleName, req.DirectionID, req.Rate, req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.update_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.update_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return nil, errors.New(locales.GetMessage("errors.teachers.not_found"))
	}

	return &Teacher{
		ID:          req.ID,
		FirstName:   req.FirstName,
		LastName:    req.LastName,
		MiddleName:  req.MiddleName,
		DirectionID: req.DirectionID,
		Rate:        req.Rate,
		IsArchived:  false,
	}, nil
}

// Delete мягко удаляет преподавателя
func (r *TeachersRepository) Delete(id int64) error {
	query := `UPDATE teachers SET isArchived = TRUE WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.teachers.delete_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.teachers.delete_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return errors.New(locales.GetMessage("errors.teachers.not_found"))
	}

	return nil
}

// Exists проверяет существование преподавателя по ID
func (r *TeachersRepository) Exists(id int64) (bool, error) {
	query := `SELECT 1 FROM teachers WHERE id = ?`

	var exists int
	err := r.db.QueryRow(query, id).Scan(&exists)
	if err != nil {
		if err == sql.ErrNoRows {
			return false, nil
		}
		return false, fmt.Errorf(locales.GetMessage("errors.teachers.exists_check_failed")+": %w", err)
	}

	return true, nil
}

// GetLessonsCount получает количество занятий у преподавателя
func (r *TeachersRepository) GetLessonsCount(teacherID int64) (int, error) {
	query := `SELECT COUNT(*) FROM lesson_teachers WHERE teacher_id = ?`

	var count int
	err := r.db.QueryRow(query, teacherID).Scan(&count)
	if err != nil {
		return 0, fmt.Errorf(locales.GetMessage("errors.teachers.get_lessons_count_failed")+": %w", err)
	}

	return count, nil
}

// Restore восстанавливает преподавателя из архива
func (r *TeachersRepository) Restore(id int64) error {
	query := `UPDATE teachers SET isArchived = FALSE WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.teachers.update_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.teachers.update_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return errors.New(locales.GetMessage("errors.teachers.not_found"))
	}

	return nil
}
