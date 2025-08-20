package storage

import (
	"database/sql"
	"fmt"
	"time"
	"tulsu-pe-timetable/backend/locales"
)

// LessonsRepository репозиторий для работы с занятиями
type LessonsRepository struct {
	db *sql.DB
}

// NewLessonsRepository создает новый репозиторий занятий
func NewLessonsRepository(db *sql.DB) *LessonsRepository {
	return &LessonsRepository{db: db}
}

// Create создает новое занятие
func (r *LessonsRepository) Create(req CreateLessonRequest) (*Lesson, error) {
	query := `
		INSERT INTO lessons (faculty_id, direction_id, teacher_id, day_of_week, lesson_number, created_at, updated_at)
		VALUES (?, ?, ?, ?, ?, ?, ?)
	`
	
	now := time.Now()
	result, err := r.db.Exec(query, req.FacultyID, req.DirectionID, req.TeacherID, req.DayOfWeek, req.LessonNumber, now, now)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.create_failed")+": %w", err)
	}
	
	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_id_failed")+": %w", err)
	}
	
	return &Lesson{
		ID:           id,
		FacultyID:    req.FacultyID,
		DirectionID:  req.DirectionID,
		TeacherID:    req.TeacherID,
		DayOfWeek:    req.DayOfWeek,
		LessonNumber: req.LessonNumber,
		CreatedAt:    now,
		UpdatedAt:    now,
	}, nil
}

// GetByID получает занятие по ID
func (r *LessonsRepository) GetByID(id int64) (*Lesson, error) {
	query := `
		SELECT l.id, l.faculty_id, l.direction_id, l.teacher_id, l.day_of_week, l.lesson_number, 
		       l.created_at, l.updated_at,
		       f.name as faculty_name, d.name as direction_name,
		       t.last_name || ' ' || t.first_name as teacher_name
		FROM lessons l
		LEFT JOIN faculties f ON l.faculty_id = f.id
		LEFT JOIN directions d ON l.direction_id = d.id
		LEFT JOIN teachers t ON l.teacher_id = t.id
		WHERE l.id = ?
	`
	
	var lesson Lesson
	err := r.db.QueryRow(query, id).Scan(
		&lesson.ID,
		&lesson.FacultyID,
		&lesson.DirectionID,
		&lesson.TeacherID,
		&lesson.DayOfWeek,
		&lesson.LessonNumber,
		&lesson.CreatedAt,
		&lesson.UpdatedAt,
		&lesson.FacultyName,
		&lesson.DirectionName,
		&lesson.TeacherName,
	)
	
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_failed")+": %w", err)
	}
	
	return &lesson, nil
}

// GetAll получает все занятия с связанными данными
func (r *LessonsRepository) GetAll() ([]Lesson, error) {
	query := `
		SELECT l.id, l.faculty_id, l.direction_id, l.teacher_id, l.day_of_week, l.lesson_number, 
		       l.created_at, l.updated_at,
		       f.name as faculty_name, d.name as direction_name,
		       t.last_name || ' ' || t.first_name as teacher_name
		FROM lessons l
		LEFT JOIN faculties f ON l.faculty_id = f.id
		LEFT JOIN directions d ON l.direction_id = d.id
		LEFT JOIN teachers t ON l.teacher_id = t.id
		ORDER BY l.day_of_week, l.lesson_number, f.name
	`
	
	rows, err := r.db.Query(query)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_all_failed")+": %w", err)
	}
	defer rows.Close()
	
	var lessons []Lesson
	for rows.Next() {
		var lesson Lesson
		err := rows.Scan(
			&lesson.ID,
			&lesson.FacultyID,
			&lesson.DirectionID,
			&lesson.TeacherID,
			&lesson.DayOfWeek,
			&lesson.LessonNumber,
			&lesson.CreatedAt,
			&lesson.UpdatedAt,
			&lesson.FacultyName,
			&lesson.DirectionName,
			&lesson.TeacherName,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.scan_failed")+": %w", err)
		}
		lessons = append(lessons, lesson)
	}
	
	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.iterate_failed")+": %w", err)
	}
	
	return lessons, nil
}

// Update обновляет занятие
func (r *LessonsRepository) Update(req UpdateLessonRequest) (*Lesson, error) {
	query := `
		UPDATE lessons
		SET faculty_id = ?, direction_id = ?, teacher_id = ?, day_of_week = ?, lesson_number = ?, updated_at = ?
		WHERE id = ?
	`
	
	now := time.Now()
	result, err := r.db.Exec(query, req.FacultyID, req.DirectionID, req.TeacherID, req.DayOfWeek, req.LessonNumber, now, req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.update_failed")+": %w", err)
	}
	
	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.update_check_failed")+": %w", err)
	}
	
	if rowsAffected == 0 {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.not_found"))
	}
	
	return &Lesson{
		ID:           req.ID,
		FacultyID:    req.FacultyID,
		DirectionID:  req.DirectionID,
		TeacherID:    req.TeacherID,
		DayOfWeek:    req.DayOfWeek,
		LessonNumber: req.LessonNumber,
		UpdatedAt:    now,
	}, nil
}

// Delete удаляет занятие
func (r *LessonsRepository) Delete(id int64) error {
	query := `DELETE FROM lessons WHERE id = ?`
	
	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.lessons.delete_failed")+": %w", err)
	}
	
	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.lessons.delete_check_failed")+": %w", err)
	}
	
	if rowsAffected == 0 {
		return fmt.Errorf(locales.GetMessage("errors.lessons.not_found"))
	}
	
	return nil
}

// Exists проверяет существование занятия по ID
func (r *LessonsRepository) Exists(id int64) (bool, error) {
	query := `SELECT 1 FROM lessons WHERE id = ?`
	
	var exists int
	err := r.db.QueryRow(query, id).Scan(&exists)
	if err != nil {
		if err == sql.ErrNoRows {
			return false, nil
		}
		return false, fmt.Errorf(locales.GetMessage("errors.lessons.exists_check_failed")+": %w", err)
	}
	
	return true, nil
}
