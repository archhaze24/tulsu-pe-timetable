package app_services

import (
	"context"
	"fmt"
	"tulsu-pe-timetable/backend/config"
	"tulsu-pe-timetable/backend/locales"
	"tulsu-pe-timetable/backend/storage"
)

// App struct
type App struct {
	ctx      context.Context
	config   *config.Config
	storage  *storage.Storage
}

// NewApp creates a new App application struct
func NewApp(cfg *config.Config) (*App, error) {
	// Инициализируем storage
	storage, err := storage.NewStorage(cfg)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize storage: %w", err)
	}
	
	return &App{
		config:  cfg,
		storage: storage,
	}, nil
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) Startup(ctx context.Context) {
	a.ctx = ctx
}

// shutdown is called when the app shuts down
func (a *App) Shutdown(ctx context.Context) {
	if a.storage != nil {
		a.storage.Close()
	}
}

type ApiResponse[T any] struct {
	Data  T      `json:"data"`
	Error string `json:"error"`
}

// GetConfig возвращает текущую конфигурацию
func (a *App) GetConfig() ApiResponse[*config.Config] {
	return ApiResponse[*config.Config]{
		Data:  a.config,
		Error: "",
	}
}

// UpdateConfig обновляет конфигурацию
func (a *App) UpdateConfig(newConfig *config.Config) ApiResponse[bool] {
	// Сохраняем новую конфигурацию
	if err := config.SaveConfig(newConfig); err != nil {
		return ApiResponse[bool]{
			Data:  false,
			Error: fmt.Sprintf("%s: %v", locales.GetMessage("errors.config.save_failed"), err),
		}
	}
	
	// Обновляем конфигурацию в состоянии
	a.config = newConfig
	
	return ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
}

// GetConfigPath возвращает путь к файлу конфигурации
func (a *App) GetConfigPath() ApiResponse[string] {
	path, err := config.GetConfigPath()
	if err != nil {
		return ApiResponse[string]{
			Data:  "",
			Error: fmt.Sprintf("%s: %v", locales.GetMessage("errors.config.path_failed"), err),
		}
	}
	
	return ApiResponse[string]{
		Data:  path,
		Error: "",
	}
}

// ===== Directions API =====

// CreateDirection создает новое направление
func (a *App) CreateDirection(req storage.CreateDirectionRequest) ApiResponse[*storage.Direction] {
	direction, err := a.storage.Directions.Create(req)
	if err != nil {
		return ApiResponse[*storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Direction]{
		Data:  direction,
		Error: "",
	}
}

// GetDirections получает все направления
func (a *App) GetDirections() ApiResponse[[]storage.Direction] {
	directions, err := a.storage.Directions.GetAll()
	if err != nil {
		return ApiResponse[[]storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[[]storage.Direction]{
		Data:  directions,
		Error: "",
	}
}

// UpdateDirection обновляет направление
func (a *App) UpdateDirection(req storage.UpdateDirectionRequest) ApiResponse[*storage.Direction] {
	direction, err := a.storage.Directions.Update(req)
	if err != nil {
		return ApiResponse[*storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Direction]{
		Data:  direction,
		Error: "",
	}
}

// DeleteDirection удаляет направление
func (a *App) DeleteDirection(id int64) ApiResponse[bool] {
	err := a.storage.Directions.Delete(id)
	if err != nil {
		return ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
}

// ===== Faculties API =====

// CreateFaculty создает новый факультет
func (a *App) CreateFaculty(req storage.CreateFacultyRequest) ApiResponse[*storage.Faculty] {
	faculty, err := a.storage.Faculties.Create(req)
	if err != nil {
		return ApiResponse[*storage.Faculty]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Faculty]{
		Data:  faculty,
		Error: "",
	}
}

// GetFaculties получает все факультеты
func (a *App) GetFaculties() ApiResponse[[]storage.Faculty] {
	faculties, err := a.storage.Faculties.GetAll()
	if err != nil {
		return ApiResponse[[]storage.Faculty]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[[]storage.Faculty]{
		Data:  faculties,
		Error: "",
	}
}

// UpdateFaculty обновляет факультет
func (a *App) UpdateFaculty(req storage.UpdateFacultyRequest) ApiResponse[*storage.Faculty] {
	faculty, err := a.storage.Faculties.Update(req)
	if err != nil {
		return ApiResponse[*storage.Faculty]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Faculty]{
		Data:  faculty,
		Error: "",
	}
}

// DeleteFaculty удаляет факультет
func (a *App) DeleteFaculty(id int64) ApiResponse[bool] {
	err := a.storage.Faculties.Delete(id)
	if err != nil {
		return ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
}

// ===== Teachers API =====

// CreateTeacher создает нового преподавателя
func (a *App) CreateTeacher(req storage.CreateTeacherRequest) ApiResponse[*storage.Teacher] {
	teacher, err := a.storage.Teachers.Create(req)
	if err != nil {
		return ApiResponse[*storage.Teacher]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Teacher]{
		Data:  teacher,
		Error: "",
	}
}

// GetTeachers получает всех преподавателей
func (a *App) GetTeachers() ApiResponse[[]storage.Teacher] {
	teachers, err := a.storage.Teachers.GetAll()
	if err != nil {
		return ApiResponse[[]storage.Teacher]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[[]storage.Teacher]{
		Data:  teachers,
		Error: "",
	}
}

// UpdateTeacher обновляет преподавателя
func (a *App) UpdateTeacher(req storage.UpdateTeacherRequest) ApiResponse[*storage.Teacher] {
	teacher, err := a.storage.Teachers.Update(req)
	if err != nil {
		return ApiResponse[*storage.Teacher]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Teacher]{
		Data:  teacher,
		Error: "",
	}
}

// DeleteTeacher удаляет преподавателя
func (a *App) DeleteTeacher(id int64) ApiResponse[bool] {
	err := a.storage.Teachers.Delete(id)
	if err != nil {
		return ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
}

// ===== Lessons API =====

// CreateLesson создает новое занятие
func (a *App) CreateLesson(req storage.CreateLessonRequest) ApiResponse[*storage.Lesson] {
	lesson, err := a.storage.Lessons.Create(req)
	if err != nil {
		return ApiResponse[*storage.Lesson]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Lesson]{
		Data:  lesson,
		Error: "",
	}
}

// GetLessons получает все занятия
func (a *App) GetLessons() ApiResponse[[]storage.Lesson] {
	lessons, err := a.storage.Lessons.GetAll()
	if err != nil {
		return ApiResponse[[]storage.Lesson]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[[]storage.Lesson]{
		Data:  lessons,
		Error: "",
	}
}

// UpdateLesson обновляет занятие
func (a *App) UpdateLesson(req storage.UpdateLessonRequest) ApiResponse[*storage.Lesson] {
	lesson, err := a.storage.Lessons.Update(req)
	if err != nil {
		return ApiResponse[*storage.Lesson]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[*storage.Lesson]{
		Data:  lesson,
		Error: "",
	}
}

// DeleteLesson удаляет занятие
func (a *App) DeleteLesson(id int64) ApiResponse[bool] {
	err := a.storage.Lessons.Delete(id)
	if err != nil {
		return ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
	}
	
	return ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
}
