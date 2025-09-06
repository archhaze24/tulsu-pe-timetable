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
	ctx     context.Context
	config  *config.Config
	storage *storage.Storage
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

// GetDirectionsByArchived получает направления по признаку архивации (по умолчанию неархивные)
func (a *App) GetDirectionsByArchived(isArchived bool) ApiResponse[[]storage.Direction] {
	directions, err := a.storage.Directions.GetAllByArchived(isArchived)
	if err != nil {
		return ApiResponse[[]storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
	}
	return ApiResponse[[]storage.Direction]{Data: directions, Error: ""}
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

// RestoreDirection восстанавливает направление
func (a *App) RestoreDirection(id int64) ApiResponse[bool] {
	if err := a.storage.Directions.Restore(id); err != nil {
		return ApiResponse[bool]{Data: false, Error: err.Error()}
	}
	return ApiResponse[bool]{Data: true, Error: ""}
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

// GetFacultiesByArchived получает факультеты по признаку архивации
func (a *App) GetFacultiesByArchived(isArchived bool) ApiResponse[[]storage.Faculty] {
	faculties, err := a.storage.Faculties.GetAllByArchived(isArchived)
	if err != nil {
		return ApiResponse[[]storage.Faculty]{Data: nil, Error: err.Error()}
	}
	return ApiResponse[[]storage.Faculty]{Data: faculties, Error: ""}
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

// RestoreFaculty восстанавливает факультет
func (a *App) RestoreFaculty(id int64) ApiResponse[bool] {
	if err := a.storage.Faculties.Restore(id); err != nil {
		return ApiResponse[bool]{Data: false, Error: err.Error()}
	}
	return ApiResponse[bool]{Data: true, Error: ""}
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

// GetTeachersByArchived получает преподавателей по признаку архивации
func (a *App) GetTeachersByArchived(isArchived bool) ApiResponse[[]storage.Teacher] {
	teachers, err := a.storage.Teachers.GetAllByArchived(isArchived)
	if err != nil {
		return ApiResponse[[]storage.Teacher]{Data: nil, Error: err.Error()}
	}
	return ApiResponse[[]storage.Teacher]{Data: teachers, Error: ""}
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

// RestoreTeacher восстанавливает преподавателя
func (a *App) RestoreTeacher(id int64) ApiResponse[bool] {
	if err := a.storage.Teachers.Restore(id); err != nil {
		return ApiResponse[bool]{Data: false, Error: err.Error()}
	}
	return ApiResponse[bool]{Data: true, Error: ""}
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

// ===== Semesters API =====

// CreateSemester создает новый семестр
func (a *App) CreateSemester(req storage.CreateSemesterRequest) ApiResponse[*storage.Semester] {
	semester, err := a.storage.Semesters.Create(req)
	if err != nil {
		return ApiResponse[*storage.Semester]{
			Data:  nil,
			Error: err.Error(),
		}
	}

	return ApiResponse[*storage.Semester]{
		Data:  semester,
		Error: "",
	}
}

// GetSemesters получает все семестры
func (a *App) GetSemesters() ApiResponse[[]storage.Semester] {
	semesters, err := a.storage.Semesters.GetAll()
	if err != nil {
		return ApiResponse[[]storage.Semester]{
			Data:  nil,
			Error: err.Error(),
		}
	}

	return ApiResponse[[]storage.Semester]{
		Data:  semesters,
		Error: "",
	}
}

// UpdateSemester обновляет семестр
func (a *App) UpdateSemester(req storage.UpdateSemesterRequest) ApiResponse[*storage.Semester] {
	semester, err := a.storage.Semesters.Update(req)
	if err != nil {
		return ApiResponse[*storage.Semester]{
			Data:  nil,
			Error: err.Error(),
		}
	}

	return ApiResponse[*storage.Semester]{
		Data:  semester,
		Error: "",
	}
}

// DeleteSemester удаляет семестр
func (a *App) DeleteSemester(id int64) ApiResponse[bool] {
	if err := a.storage.Semesters.Delete(id); err != nil {
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

// ===== Semesters API =====

// GetSemesters получает все семестры
func (a *App) GetSemesters() ApiResponse[[]storage.Semester] {
	semesters, err := a.storage.Semesters.GetAll()
	if err != nil {
		return ApiResponse[[]storage.Semester]{Data: nil, Error: err.Error()}
	}
	return ApiResponse[[]storage.Semester]{Data: semesters, Error: ""}
}

// GetSemestersByArchived получает семестры по признаку архивации
func (a *App) GetSemestersByArchived(isArchived bool) ApiResponse[[]storage.Semester] {
	semesters, err := a.storage.Semesters.GetAllByArchived(isArchived)
	if err != nil {
		return ApiResponse[[]storage.Semester]{Data: nil, Error: err.Error()}
	}
	return ApiResponse[[]storage.Semester]{Data: semesters, Error: ""}
}

// CreateSemester создает новый семестр
func (a *App) CreateSemester(req storage.CreateSemesterRequest) ApiResponse[*storage.Semester] {
	semester, err := a.storage.Semesters.Create(req)
	if err != nil {
		return ApiResponse[*storage.Semester]{Data: nil, Error: err.Error()}
	}
	return ApiResponse[*storage.Semester]{Data: semester, Error: ""}
}

// UpdateSemester обновляет семестр
func (a *App) UpdateSemester(req storage.UpdateSemesterRequest) ApiResponse[*storage.Semester] {
	semester, err := a.storage.Semesters.Update(req)
	if err != nil {
		return ApiResponse[*storage.Semester]{Data: nil, Error: err.Error()}
	}
	return ApiResponse[*storage.Semester]{Data: semester, Error: ""}
}

// DeleteSemester мягко удаляет семестр
func (a *App) DeleteSemester(id int64) ApiResponse[bool] {
	if err := a.storage.Semesters.Delete(id); err != nil {
		return ApiResponse[bool]{Data: false, Error: err.Error()}
	}
	return ApiResponse[bool]{Data: true, Error: ""}
}

// RestoreSemester восстанавливает семестр
func (a *App) RestoreSemester(id int64) ApiResponse[bool] {
	if err := a.storage.Semesters.Restore(id); err != nil {
		return ApiResponse[bool]{Data: false, Error: err.Error()}
	}
	return ApiResponse[bool]{Data: true, Error: ""}
}
