package storage

import (
	"time"
)

// Direction представляет направление (футбол, плавание и т.д.)
type Direction struct {
	ID          int64     `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}

// Faculty представляет факультет
type Faculty struct {
	ID        int64     `json:"id"`
	Name      string    `json:"name"`
	ShortName string    `json:"short_name"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

// Teacher представляет преподавателя
type Teacher struct {
	ID         int64     `json:"id"`
	LastName   string    `json:"last_name"`
	FirstName  string    `json:"first_name"`
	MiddleName string    `json:"middle_name"`
	Rate       float64   `json:"rate"`
	CreatedAt  time.Time `json:"created_at"`
	UpdatedAt  time.Time `json:"updated_at"`
}

// Lesson представляет занятие в расписании
type Lesson struct {
	ID           int64     `json:"id"`
	FacultyID    int64     `json:"faculty_id"`
	DirectionID  int64     `json:"direction_id"`
	TeacherID    *int64    `json:"teacher_id,omitempty"`
	DayOfWeek    int       `json:"day_of_week"`
	LessonNumber int       `json:"lesson_number"`
	CreatedAt    time.Time `json:"created_at"`
	UpdatedAt    time.Time `json:"updated_at"`
	
	// Связанные данные (заполняются при запросах с JOIN)
	FacultyName   string  `json:"faculty_name,omitempty"`
	DirectionName string  `json:"direction_name,omitempty"`
	TeacherName   string  `json:"teacher_name,omitempty"`
}

// CreateDirectionRequest запрос на создание направления
type CreateDirectionRequest struct {
	Name        string `json:"name"`
	Description string `json:"description"`
}

// UpdateDirectionRequest запрос на обновление направления
type UpdateDirectionRequest struct {
	ID          int64  `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
}

// CreateFacultyRequest запрос на создание факультета
type CreateFacultyRequest struct {
	Name      string `json:"name"`
	ShortName string `json:"short_name"`
}

// UpdateFacultyRequest запрос на обновление факультета
type UpdateFacultyRequest struct {
	ID        int64  `json:"id"`
	Name      string `json:"name"`
	ShortName string `json:"short_name"`
}

// CreateTeacherRequest запрос на создание преподавателя
type CreateTeacherRequest struct {
	LastName   string  `json:"last_name"`
	FirstName  string  `json:"first_name"`
	MiddleName string  `json:"middle_name"`
	Rate       float64 `json:"rate"`
}

// UpdateTeacherRequest запрос на обновление преподавателя
type UpdateTeacherRequest struct {
	ID         int64   `json:"id"`
	LastName   string  `json:"last_name"`
	FirstName  string  `json:"first_name"`
	MiddleName string  `json:"middle_name"`
	Rate       float64 `json:"rate"`
}

// CreateLessonRequest запрос на создание занятия
type CreateLessonRequest struct {
	FacultyID    int64  `json:"faculty_id"`
	DirectionID  int64  `json:"direction_id"`
	TeacherID    *int64 `json:"teacher_id,omitempty"`
	DayOfWeek    int    `json:"day_of_week"`
	LessonNumber int    `json:"lesson_number"`
}

// UpdateLessonRequest запрос на обновление занятия
type UpdateLessonRequest struct {
	ID           int64  `json:"id"`
	FacultyID    int64  `json:"faculty_id"`
	DirectionID  int64  `json:"direction_id"`
	TeacherID    *int64 `json:"teacher_id,omitempty"`
	DayOfWeek    int    `json:"day_of_week"`
	LessonNumber int    `json:"lesson_number"`
}
