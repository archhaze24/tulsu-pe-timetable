package storage

import (
	"database/sql"
	"fmt"
	"time"
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
		INSERT INTO teachers (last_name, first_name, middle_name, rate, created_at, updated_at)
		VALUES (?, ?, ?, ?, ?, ?)
	`
	
	now := time.Now()
	result, err := r.db.Exec(query, req.LastName, req.FirstName, req.MiddleName, req.Rate, now, now)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.create_failed")+": %w", err)
	}
	
	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.get_id_failed")+": %w", err)
	}
	
	return &Teacher{
		ID:         id,
		LastName:   req.LastName,
		FirstName:  req.FirstName,
		MiddleName: req.MiddleName,
		Rate:       req.Rate,
		CreatedAt:  now,
		UpdatedAt:  now,
	}, nil
}

// GetByID получает преподавателя по ID
func (r *TeachersRepository) GetByID(id int64) (*Teacher, error) {
	query := `
		SELECT id, last_name, first_name, middle_name, rate, created_at, updated_at
		FROM teachers
		WHERE id = ?
	`
	
	var teacher Teacher
	err := r.db.QueryRow(query, id).Scan(
		&teacher.ID,
		&teacher.LastName,
		&teacher.FirstName,
		&teacher.MiddleName,
		&teacher.Rate,
		&teacher.CreatedAt,
		&teacher.UpdatedAt,
	)
	
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf(locales.GetMessage("errors.teachers.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.get_failed")+": %w", err)
	}
	
	return &teacher, nil
}

// GetAll получает всех преподавателей
func (r *TeachersRepository) GetAll() ([]Teacher, error) {
	query := `
		SELECT id, last_name, first_name, middle_name, rate, created_at, updated_at
		FROM teachers
		ORDER BY last_name, first_name
	`
	
	rows, err := r.db.Query(query)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.get_all_failed")+": %w", err)
	}
	defer rows.Close()
	
	var teachers []Teacher
	for rows.Next() {
		var teacher Teacher
		err := rows.Scan(
			&teacher.ID,
			&teacher.LastName,
			&teacher.FirstName,
			&teacher.MiddleName,
			&teacher.Rate,
			&teacher.CreatedAt,
			&teacher.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.teachers.scan_failed")+": %w", err)
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
		SET last_name = ?, first_name = ?, middle_name = ?, rate = ?, updated_at = ?
		WHERE id = ?
	`
	
	now := time.Now()
	result, err := r.db.Exec(query, req.LastName, req.FirstName, req.MiddleName, req.Rate, now, req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.update_failed")+": %w", err)
	}
	
	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.update_check_failed")+": %w", err)
	}
	
	if rowsAffected == 0 {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.not_found"))
	}
	
	return &Teacher{
		ID:         req.ID,
		LastName:   req.LastName,
		FirstName:  req.FirstName,
		MiddleName: req.MiddleName,
		Rate:       req.Rate,
		UpdatedAt:  now,
	}, nil
}

// Delete удаляет преподавателя
func (r *TeachersRepository) Delete(id int64) error {
	query := `DELETE FROM teachers WHERE id = ?`
	
	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.teachers.delete_failed")+": %w", err)
	}
	
	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.teachers.delete_check_failed")+": %w", err)
	}
	
	if rowsAffected == 0 {
		return fmt.Errorf(locales.GetMessage("errors.teachers.not_found"))
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
	query := `SELECT COUNT(*) FROM lessons WHERE teacher_id = ?`
	
	var count int
	err := r.db.QueryRow(query, teacherID).Scan(&count)
	if err != nil {
		return 0, fmt.Errorf(locales.GetMessage("errors.teachers.get_lessons_count_failed")+": %w", err)
	}
	
	return count, nil
}
