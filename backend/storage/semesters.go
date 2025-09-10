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

// Create создает новый семестр и автоматически привязывает всех негостевых преподавателей
func (r *SemestersRepository) Create(req CreateSemesterRequest) (*Semester, error) {
	// Начинаем транзакцию
	tx, err := r.db.Begin()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.create_failed")+": %w", err)
	}
	defer tx.Rollback()

	// Создаем семестр
	query := `
		INSERT INTO semesters (name, start_date, end_date)
		VALUES (?, ?, ?)
	`

	result, err := tx.Exec(query, req.Name, req.StartDate, req.EndDate)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.create_failed")+": %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.get_id_failed")+": %w", err)
	}

	// Привязываем всех негостевых неархивированных преподавателей к семестру
	err = r.bindNonGuestTeachersToSemester(tx, id)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.bind_teachers_failed")+": %w", err)
	}

	// Подтверждаем транзакцию
	if err := tx.Commit(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.lessons.commit_failed")+": %w", err)
	}

	return &Semester{
		ID:         id,
		Name:       req.Name,
		StartDate:  req.StartDate,
		EndDate:    req.EndDate,
		IsArchived: false,
	}, nil
}

// GetByID получает семестр по ID
func (r *SemestersRepository) GetByID(id int64) (*Semester, error) {
	query := `
		SELECT id, name, start_date, end_date, isArchived
		FROM semesters
		WHERE id = ?
	`

	var semester Semester
	err := r.db.QueryRow(query, id).Scan(
		&semester.ID,
		&semester.Name,
		&semester.StartDate,
		&semester.EndDate,
		&semester.IsArchived,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf("%s", locales.GetMessage("errors.semesters.not_found"))
		}
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.get_failed")+": %w", err)
	}

	return &semester, nil
}

// GetAll получает все семестры
func (r *SemestersRepository) GetAll() ([]Semester, error) {
	return r.GetAllByArchived(false)
}

// GetAllByArchived получает семестры по признаку архива
func (r *SemestersRepository) GetAllByArchived(isArchived bool) ([]Semester, error) {
	query := `
		SELECT id, name, start_date, end_date, isArchived
		FROM semesters
		WHERE isArchived = ?
		ORDER BY start_date DESC
	`

	rows, err := r.db.Query(query, isArchived)
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
			&semester.IsArchived,
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
		return nil, fmt.Errorf("%s", locales.GetMessage("errors.semesters.not_found"))
	}

	return &Semester{
		ID:         req.ID,
		Name:       req.Name,
		StartDate:  req.StartDate,
		EndDate:    req.EndDate,
		IsArchived: false,
	}, nil
}

// Delete мягко удаляет семестр
func (r *SemestersRepository) Delete(id int64) error {
	query := `UPDATE semesters SET isArchived = TRUE WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.delete_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.delete_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf("%s", locales.GetMessage("errors.semesters.not_found"))
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

// Restore восстанавливает семестр из архива
func (r *SemestersRepository) Restore(id int64) error {
	query := `UPDATE semesters SET isArchived = FALSE WHERE id = ?`

	result, err := r.db.Exec(query, id)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.update_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.update_check_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf("%s", locales.GetMessage("errors.semesters.not_found"))
	}

	return nil
}

// bindNonGuestTeachersToSemester привязывает всех негостевых неархивированных преподавателей к семестру
func (r *SemestersRepository) bindNonGuestTeachersToSemester(tx *sql.Tx, semesterID int64) error {
	query := `
		INSERT INTO semester_teachers (semester_id, teacher_id)
		SELECT ?, t.id
		FROM teachers t
		WHERE t.isArchived = FALSE AND t.isGuest = FALSE
	`

	_, err := tx.Exec(query, semesterID)
	return err
}

// GetSemesterTeachers получает всех преподавателей семестра
func (r *SemestersRepository) GetSemesterTeachers(semesterID int64) ([]Teacher, error) {
	query := `
		SELECT t.id, t.first_name, t.last_name, t.middle_name, t.direction_id, t.rate, t.isArchived, t.isGuest,
		       d.name as direction_name
		FROM teachers t
		LEFT JOIN directions d ON t.direction_id = d.id
		INNER JOIN semester_teachers st ON t.id = st.teacher_id
		WHERE st.semester_id = ?
		ORDER BY t.last_name, t.first_name
	`

	rows, err := r.db.Query(query, semesterID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.get_teachers_failed")+": %w", err)
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
			&teacher.IsArchived,
			&teacher.IsGuest,
			&directionName,
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

// BindTeacher привязывает преподавателя к семестру
func (r *SemestersRepository) BindTeacher(req BindTeacherToSemesterRequest) error {
	// Проверяем существование семестра
	exists, err := r.Exists(req.SemesterID)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.exists_check_failed")+": %w", err)
	}
	if !exists {
		return fmt.Errorf("%s", locales.GetMessage("errors.semesters.not_found"))
	}

	// Проверяем существование преподавателя
	query := `SELECT 1 FROM teachers WHERE id = ?`
	var teacherExists int
	err = r.db.QueryRow(query, req.TeacherID).Scan(&teacherExists)
	if err != nil {
		if err == sql.ErrNoRows {
			return fmt.Errorf("%s", locales.GetMessage("errors.teachers.not_found"))
		}
		return fmt.Errorf(locales.GetMessage("errors.teachers.exists_check_failed")+": %w", err)
	}

	// Проверяем, не привязан ли уже преподаватель к семестру
	checkQuery := `SELECT 1 FROM semester_teachers WHERE semester_id = ? AND teacher_id = ?`
	var alreadyBound int
	err = r.db.QueryRow(checkQuery, req.SemesterID, req.TeacherID).Scan(&alreadyBound)
	if err == nil {
		// Уже привязан - возвращаем успех
		return nil
	}
	if err != sql.ErrNoRows {
		return fmt.Errorf(locales.GetMessage("errors.semesters.bind_teacher_failed")+": %w", err)
	}

	// Привязываем преподавателя к семестру
	insertQuery := `INSERT INTO semester_teachers (semester_id, teacher_id) VALUES (?, ?)`
	_, err = r.db.Exec(insertQuery, req.SemesterID, req.TeacherID)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.bind_teacher_failed")+": %w", err)
	}

	return nil
}

// UnbindTeacher отвязывает преподавателя от семестра
func (r *SemestersRepository) UnbindTeacher(req UnbindTeacherFromSemesterRequest) error {
	query := `DELETE FROM semester_teachers WHERE semester_id = ? AND teacher_id = ?`

	result, err := r.db.Exec(query, req.SemesterID, req.TeacherID)
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.unbind_teacher_failed")+": %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.semesters.unbind_teacher_failed")+": %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf("%s", locales.GetMessage("errors.teachers.not_found"))
	}

	return nil
}

// GetAllTeachersForSemester получает всех преподавателей доступных для семестра (включая гостевых и архивированных)
func (r *SemestersRepository) GetAllTeachersForSemester(semesterID int64) ([]Teacher, error) {
	query := `
		SELECT t.id, t.first_name, t.last_name, t.middle_name, t.direction_id, t.rate, t.isArchived, t.isGuest,
		       d.name as direction_name,
		       CASE WHEN st.teacher_id IS NOT NULL THEN 1 ELSE 0 END as is_bound
		FROM teachers t
		LEFT JOIN directions d ON t.direction_id = d.id
		LEFT JOIN semester_teachers st ON t.id = st.teacher_id AND st.semester_id = ?
		ORDER BY t.isArchived, t.last_name, t.first_name
	`

	rows, err := r.db.Query(query, semesterID)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.semesters.get_teachers_failed")+": %w", err)
	}
	defer rows.Close()

	var teachers []Teacher
	for rows.Next() {
		var teacher Teacher
		var directionName sql.NullString
		var isBound int
		err := rows.Scan(
			&teacher.ID,
			&teacher.FirstName,
			&teacher.LastName,
			&teacher.MiddleName,
			&teacher.DirectionID,
			&teacher.Rate,
			&teacher.IsArchived,
			&teacher.IsGuest,
			&directionName,
			&isBound,
		)
		if err != nil {
			return nil, fmt.Errorf(locales.GetMessage("errors.teachers.scan_failed")+": %w", err)
		}

		if directionName.Valid {
			teacher.DirectionName = directionName.String
		}

		teacher.IsBound = isBound == 1

		teachers = append(teachers, teacher)
	}

	if err = rows.Err(); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.teachers.iterate_failed")+": %w", err)
	}

	return teachers, nil
}
