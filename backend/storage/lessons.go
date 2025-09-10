package storage

import (
	"database/sql"
	"fmt"
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
	// Начинаем транзакцию
	tx, err := r.db.Begin()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.create_failed")+": %w", err)
	}
	defer tx.Rollback()

	// Создаем занятие
	query := `
		INSERT INTO lessons (semester_id, day_of_week, start_time, end_time, direction_id, teacher_count)
		VALUES (?, ?, ?, ?, ?, ?)
	`

	result, err := tx.Exec(query, req.SemesterID, req.DayOfWeek, req.StartTime, req.EndTime, req.DirectionID, req.TeacherCount)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.create_failed")+": %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_id_failed")+": %w", err)
	}

	// Добавляем связи с факультетами
	for _, facultyID := range req.FacultyIDs {
		_, err := tx.Exec("INSERT INTO lesson_faculties (lesson_id, faculty_id) VALUES (?, ?)", id, facultyID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.create_faculty_link_failed")+": %w", err)
		}
	}

	// Добавляем связи с преподавателями
	for _, teacherID := range req.TeacherIDs {
		_, err := tx.Exec("INSERT INTO lesson_teachers (lesson_id, teacher_id) VALUES (?, ?)", id, teacherID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.create_teacher_link_failed")+": %w", err)
		}
	}

	// Подтверждаем транзакцию
	if err := tx.Commit(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.commit_failed")+": %w", err)
	}

	return &Lesson{
		ID:           id,
		SemesterID:   req.SemesterID,
		DayOfWeek:    req.DayOfWeek,
		StartTime:    req.StartTime,
		EndTime:      req.EndTime,
		DirectionID:  req.DirectionID,
		TeacherCount: req.TeacherCount,
		FacultyIDs:   req.FacultyIDs,
		TeacherIDs:   req.TeacherIDs,
	}, nil
}

// GetByID получает занятие по ID со всеми связанными данными
func (r *LessonsRepository) GetByID(id int64) (*Lesson, error) {
	// Получаем основную информацию о занятии
	query := `
		SELECT l.id, l.semester_id, l.day_of_week, l.start_time, l.end_time, l.direction_id, l.teacher_count,
		       s.name as semester_name, d.name as direction_name
		FROM lessons l
		LEFT JOIN semesters s ON l.semester_id = s.id
		LEFT JOIN directions d ON l.direction_id = d.id
		WHERE l.id = ?
	`

	var lesson Lesson
	var semesterName, directionName sql.NullString
	var teacherCount sql.NullInt64

	err := r.db.QueryRow(query, id).Scan(
		&lesson.ID,
		&lesson.SemesterID,
		&lesson.DayOfWeek,
		&lesson.StartTime,
		&lesson.EndTime,
		&lesson.DirectionID,
		&teacherCount,
		&semesterName,
		&directionName,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_failed")+": %w", err)
	}

	if teacherCount.Valid {
		count := int(teacherCount.Int64)
		lesson.TeacherCount = &count
	}

	if semesterName.Valid {
		lesson.SemesterName = semesterName.String
	}
	if directionName.Valid {
		lesson.DirectionName = directionName.String
	}

	// Получаем список факультетов
	facultyIDs, facultyNames, err := r.getLessonFaculties(id)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_faculties_failed")+": %w", err)
	}
	lesson.FacultyIDs = facultyIDs
	lesson.FacultyNames = facultyNames

	// Получаем список преподавателей
	teacherIDs, teacherNames, err := r.getLessonTeachers(id)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_teachers_failed")+": %w", err)
	}
	lesson.TeacherIDs = teacherIDs
	lesson.TeacherNames = teacherNames

	return &lesson, nil
}

// GetAll получает все занятия с связанными данными
func (r *LessonsRepository) GetAll() ([]Lesson, error) {
	query := `
		SELECT l.id, l.semester_id, l.day_of_week, l.start_time, l.end_time, l.direction_id, l.teacher_count,
		       s.name as semester_name, d.name as direction_name
		FROM lessons l
		LEFT JOIN semesters s ON l.semester_id = s.id
		LEFT JOIN directions d ON l.direction_id = d.id
		ORDER BY l.day_of_week, l.start_time, s.name
	`

	rows, err := r.db.Query(query)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_all_failed")+": %w", err)
	}
	defer rows.Close()

	var lessons []Lesson
	for rows.Next() {
		var lesson Lesson
		var semesterName, directionName sql.NullString
		var teacherCount sql.NullInt64

		err := rows.Scan(
			&lesson.ID,
			&lesson.SemesterID,
			&lesson.DayOfWeek,
			&lesson.StartTime,
			&lesson.EndTime,
			&lesson.DirectionID,
			&teacherCount,
			&semesterName,
			&directionName,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.scan_failed")+": %w", err)
		}

		if teacherCount.Valid {
			count := int(teacherCount.Int64)
			lesson.TeacherCount = &count
		}

		if semesterName.Valid {
			lesson.SemesterName = semesterName.String
		}
		if directionName.Valid {
			lesson.DirectionName = directionName.String
		}

		// Получаем связанные факультеты и преподавателей для каждого занятия
		facultyIDs, facultyNames, err := r.getLessonFaculties(lesson.ID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_faculties_failed")+": %w", err)
		}
		lesson.FacultyIDs = facultyIDs
		lesson.FacultyNames = facultyNames

		teacherIDs, teacherNames, err := r.getLessonTeachers(lesson.ID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_teachers_failed")+": %w", err)
		}
		lesson.TeacherIDs = teacherIDs
		lesson.TeacherNames = teacherNames

		lessons = append(lessons, lesson)
	}

	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.iterate_failed")+": %w", err)
	}

	return lessons, nil
}

// Update обновляет занятие со всеми связями
func (r *LessonsRepository) Update(req UpdateLessonRequest) (*Lesson, error) {
	// Начинаем транзакцию
	tx, err := r.db.Begin()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.update_failed")+": %w", err)
	}
	defer tx.Rollback()

	// Обновляем основную информацию о занятии
	query := `
		UPDATE lessons
		SET semester_id = ?, day_of_week = ?, start_time = ?, end_time = ?, direction_id = ?, teacher_count = ?
		WHERE id = ?
	`

	result, err := tx.Exec(query, req.SemesterID, req.DayOfWeek, req.StartTime, req.EndTime, req.DirectionID, req.TeacherCount, req.ID)
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

	// Удаляем старые связи с факультетами
	_, err = tx.Exec("DELETE FROM lesson_faculties WHERE lesson_id = ?", req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.delete_faculty_links_failed")+": %w", err)
	}

	// Добавляем новые связи с факультетами
	for _, facultyID := range req.FacultyIDs {
		_, err := tx.Exec("INSERT INTO lesson_faculties (lesson_id, faculty_id) VALUES (?, ?)", req.ID, facultyID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.create_faculty_link_failed")+": %w", err)
		}
	}

	// Удаляем старые связи с преподавателями
	_, err = tx.Exec("DELETE FROM lesson_teachers WHERE lesson_id = ?", req.ID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.delete_teacher_links_failed")+": %w", err)
	}

	// Добавляем новые связи с преподавателями
	for _, teacherID := range req.TeacherIDs {
		_, err := tx.Exec("INSERT INTO lesson_teachers (lesson_id, teacher_id) VALUES (?, ?)", req.ID, teacherID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.create_teacher_link_failed")+": %w", err)
		}
	}

	// Подтверждаем транзакцию
	if err := tx.Commit(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.commit_failed")+": %w", err)
	}

	return &Lesson{
		ID:           req.ID,
		SemesterID:   req.SemesterID,
		DayOfWeek:    req.DayOfWeek,
		StartTime:    req.StartTime,
		EndTime:      req.EndTime,
		DirectionID:  req.DirectionID,
		TeacherCount: req.TeacherCount,
		FacultyIDs:   req.FacultyIDs,
		TeacherIDs:   req.TeacherIDs,
	}, nil
}

// Delete удаляет занятие со всеми связями
func (r *LessonsRepository) Delete(id int64) error {
	// Начинаем транзакцию
	tx, err := r.db.Begin()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.lessons.delete_failed")+": %w", err)
	}
	defer tx.Rollback()

	// Удаляем связи с факультетами
	_, err = tx.Exec("DELETE FROM lesson_faculties WHERE lesson_id = ?", id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.lessons.delete_faculty_links_failed")+": %w", err)
	}

	// Удаляем связи с преподавателями
	_, err = tx.Exec("DELETE FROM lesson_teachers WHERE lesson_id = ?", id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.lessons.delete_teacher_links_failed")+": %w", err)
	}

	// Удаляем само занятие
	result, err := tx.Exec("DELETE FROM lessons WHERE id = ?", id)
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

	// Подтверждаем транзакцию
	if err := tx.Commit(); err != nil {
		return fmt.Errorf(locales.GetMessage("errors.lessons.commit_failed")+": %w", err)
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

// getLessonFaculties получает список факультетов для занятия
func (r *LessonsRepository) getLessonFaculties(lessonID int64) ([]int64, []string, error) {
	query := `
		SELECT f.id, f.name
		FROM lesson_faculties lf
		JOIN faculties f ON lf.faculty_id = f.id
		WHERE lf.lesson_id = ?
		ORDER BY f.name
	`

	rows, err := r.db.Query(query, lessonID)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()

	var facultyIDs []int64
	var facultyNames []string
	for rows.Next() {
		var id int64
		var name string
		err := rows.Scan(&id, &name)
		if err != nil {
			return nil, nil, err
		}
		facultyIDs = append(facultyIDs, id)
		facultyNames = append(facultyNames, name)
	}

	if err = rows.Err(); err != nil {
		return nil, nil, err
	}

	return facultyIDs, facultyNames, nil
}

// getLessonTeachers получает список преподавателей для занятия
func (r *LessonsRepository) getLessonTeachers(lessonID int64) ([]int64, []string, error) {
	query := `
		SELECT t.id, t.first_name || ' ' || t.last_name as full_name
		FROM lesson_teachers lt
		JOIN teachers t ON lt.teacher_id = t.id
		WHERE lt.lesson_id = ?
		ORDER BY t.last_name, t.first_name
	`

	rows, err := r.db.Query(query, lessonID)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()

	var teacherIDs []int64
	var teacherNames []string
	for rows.Next() {
		var id int64
		var name string
		err := rows.Scan(&id, &name)
		if err != nil {
			return nil, nil, err
		}
		teacherIDs = append(teacherIDs, id)
		teacherNames = append(teacherNames, name)
	}

	if err = rows.Err(); err != nil {
		return nil, nil, err
	}

	return teacherIDs, teacherNames, nil
}

// GetBySemesterID получает все занятия семестра со связанными данными
func (r *LessonsRepository) GetBySemesterID(semesterID int64) ([]Lesson, error) {
	query := `
		SELECT l.id, l.semester_id, l.day_of_week, l.start_time, l.end_time, l.direction_id, l.teacher_count,
		       s.name as semester_name, d.name as direction_name
		FROM lessons l
		LEFT JOIN semesters s ON l.semester_id = s.id
		LEFT JOIN directions d ON l.direction_id = d.id
		WHERE l.semester_id = ?
		ORDER BY l.day_of_week, l.start_time
	`

	rows, err := r.db.Query(query, semesterID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_all_failed")+": %w", err)
	}
	defer rows.Close()

	var lessons []Lesson
	for rows.Next() {
		var lesson Lesson
		var semesterName, directionName sql.NullString
		var teacherCount sql.NullInt64

		err := rows.Scan(
			&lesson.ID,
			&lesson.SemesterID,
			&lesson.DayOfWeek,
			&lesson.StartTime,
			&lesson.EndTime,
			&lesson.DirectionID,
			&teacherCount,
			&semesterName,
			&directionName,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.scan_failed")+": %w", err)
		}

		if teacherCount.Valid {
			count := int(teacherCount.Int64)
			lesson.TeacherCount = &count
		}

		if semesterName.Valid {
			lesson.SemesterName = semesterName.String
		}
		if directionName.Valid {
			lesson.DirectionName = directionName.String
		}

		// Получаем связанные факультеты и преподавателей для каждого занятия
		facultyIDs, facultyNames, err := r.getLessonFaculties(lesson.ID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_faculties_failed")+": %w", err)
		}
		lesson.FacultyIDs = facultyIDs
		lesson.FacultyNames = facultyNames

		teacherIDs, teacherNames, err := r.getLessonTeachers(lesson.ID)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.lessons.get_teachers_failed")+": %w", err)
		}
		lesson.TeacherIDs = teacherIDs
		lesson.TeacherNames = teacherNames

		lessons = append(lessons, lesson)
	}

	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.iterate_failed")+": %w", err)
	}

	return lessons, nil
}
