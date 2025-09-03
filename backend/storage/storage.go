package storage

import (
	"tulsu-pe-timetable/backend/config"
)

// Storage главный менеджер для работы с базой данных
type Storage struct {
	db *Database

	// Репозитории
	Directions *DirectionsRepository
	Faculties  *FacultiesRepository
	Teachers   *TeachersRepository
	Lessons    *LessonsRepository
	Semesters  *SemestersRepository
}

// NewStorage создает новый экземпляр storage менеджера
func NewStorage(cfg *config.Config) (*Storage, error) {
	// Инициализируем базу данных
	db, err := NewDatabase(cfg)
	if err != nil {
		return nil, err
	}

	// Создаем репозитории
	directionsRepo := NewDirectionsRepository(db.GetDB())
	facultiesRepo := NewFacultiesRepository(db.GetDB())
	teachersRepo := NewTeachersRepository(db.GetDB())
	lessonsRepo := NewLessonsRepository(db.GetDB())
	semestersRepo := NewSemestersRepository(db.GetDB())

	return &Storage{
		db:         db,
		Directions: directionsRepo,
		Faculties:  facultiesRepo,
		Teachers:   teachersRepo,
		Lessons:    lessonsRepo,
		Semesters:  semestersRepo,
	}, nil
}

// Close закрывает соединение с базой данных
func (s *Storage) Close() error {
	return s.db.Close()
}

// GetDB возвращает указатель на sql.DB для прямого доступа
func (s *Storage) GetDB() *Database {
	return s.db
}
