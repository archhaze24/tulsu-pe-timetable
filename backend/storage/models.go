package storage

// Direction представляет направление (футбол, плавание и т.д.)
type Direction struct {
	ID   int64  `json:"id"`
	Name string `json:"name"`
}

// Faculty представляет факультет
type Faculty struct {
	ID   int64  `json:"id"`
	Name string `json:"name"`
}

// Teacher представляет преподавателя
type Teacher struct {
	ID          int64   `json:"id"`
	FirstName   string  `json:"first_name"`
	LastName    string  `json:"last_name"`
	MiddleName  string  `json:"middle_name"`
	DirectionID int64   `json:"direction_id"`
	Rate        float64 `json:"rate"`

	// Связанные данные (заполняются при запросах с JOIN)
	DirectionName string `json:"direction_name,omitempty"`
}

// Semester представляет семестр
type Semester struct {
	ID        int64  `json:"id"`
	Name      string `json:"name"`
	StartDate string `json:"start_date"`
	EndDate   string `json:"end_date"`
}

// Lesson представляет занятие в расписании
type Lesson struct {
	ID           int64  `json:"id"`
	SemesterID   int64  `json:"semester_id"`
	DayOfWeek    int    `json:"day_of_week"`
	StartTime    string `json:"start_time"`
	EndTime      string `json:"end_time"`
	DirectionID  int64  `json:"direction_id"`
	TeacherCount *int   `json:"teacher_count,omitempty"`

	// Связанные данные (заполняются при запросах с JOIN)
	SemesterName  string   `json:"semester_name,omitempty"`
	DirectionName string   `json:"direction_name,omitempty"`
	FacultyNames  []string `json:"faculty_names,omitempty"`
	TeacherNames  []string `json:"teacher_names,omitempty"`

	// Списки связанных сущностей
	FacultyIDs []int64 `json:"faculty_ids,omitempty"`
	TeacherIDs []int64 `json:"teacher_ids,omitempty"`
}

// CreateDirectionRequest запрос на создание направления
type CreateDirectionRequest struct {
	Name string `json:"name"`
}

// UpdateDirectionRequest запрос на обновление направления
type UpdateDirectionRequest struct {
	ID   int64  `json:"id"`
	Name string `json:"name"`
}

// CreateFacultyRequest запрос на создание факультета
type CreateFacultyRequest struct {
	Name string `json:"name"`
}

// UpdateFacultyRequest запрос на обновление факультета
type UpdateFacultyRequest struct {
	ID   int64  `json:"id"`
	Name string `json:"name"`
}

// CreateTeacherRequest запрос на создание преподавателя
type CreateTeacherRequest struct {
	FirstName   string  `json:"first_name"`
	LastName    string  `json:"last_name"`
	MiddleName  string  `json:"middle_name"`
	DirectionID int64   `json:"direction_id"`
	Rate        float64 `json:"rate"`
}

// UpdateTeacherRequest запрос на обновление преподавателя
type UpdateTeacherRequest struct {
	ID          int64   `json:"id"`
	FirstName   string  `json:"first_name"`
	LastName    string  `json:"last_name"`
	MiddleName  string  `json:"middle_name"`
	DirectionID int64   `json:"direction_id"`
	Rate        float64 `json:"rate"`
}

// CreateSemesterRequest запрос на создание семестра
type CreateSemesterRequest struct {
	Name      string `json:"name"`
	StartDate string `json:"start_date"`
	EndDate   string `json:"end_date"`
}

// UpdateSemesterRequest запрос на обновление семестра
type UpdateSemesterRequest struct {
	ID        int64  `json:"id"`
	Name      string `json:"name"`
	StartDate string `json:"start_date"`
	EndDate   string `json:"end_date"`
}

// CreateLessonRequest запрос на создание занятия
type CreateLessonRequest struct {
	SemesterID   int64   `json:"semester_id"`
	DayOfWeek    int     `json:"day_of_week"`
	StartTime    string  `json:"start_time"`
	EndTime      string  `json:"end_time"`
	DirectionID  int64   `json:"direction_id"`
	TeacherCount *int    `json:"teacher_count,omitempty"`
	FacultyIDs   []int64 `json:"faculty_ids"`
	TeacherIDs   []int64 `json:"teacher_ids"`
}

// UpdateLessonRequest запрос на обновление занятия
type UpdateLessonRequest struct {
	ID           int64   `json:"id"`
	SemesterID   int64   `json:"semester_id"`
	DayOfWeek    int     `json:"day_of_week"`
	StartTime    string  `json:"start_time"`
	EndTime      string  `json:"end_time"`
	DirectionID  int64   `json:"direction_id"`
	TeacherCount *int    `json:"teacher_count,omitempty"`
	FacultyIDs   []int64 `json:"faculty_ids"`
	TeacherIDs   []int64 `json:"teacher_ids"`
}
