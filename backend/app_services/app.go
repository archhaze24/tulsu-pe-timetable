package app_services

import (
	"context"
	"fmt"
	"tulsu-pe-timetable/backend/config"
	"tulsu-pe-timetable/backend/locales"
	"tulsu-pe-timetable/backend/logger"
	"tulsu-pe-timetable/backend/storage"
)

// App struct
type App struct {
	ctx     context.Context
	config  *config.Config
	storage *storage.Storage
	logger  *logger.Logger
}

func NewApp(cfg *config.Config) (*App, error) {
	// Инициализируем storage
	storage, err := storage.NewStorage(cfg)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize storage: %w", err)
	}

	// Инициализируем логгер рядом с базой данных
	logg, err := logger.NewLogger(cfg.DbPath)
	if err != nil {
		return nil, fmt.Errorf("%s: %v", locales.GetMessage("errors.logger.init_failed"), err)
	}

	return &App{
		config:  cfg,
		storage: storage,
		logger:  logg,
	}, nil
}

func (a *App) Startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) Shutdown(ctx context.Context) {
	if a.storage != nil {
		a.storage.Close()
	}
	if a.logger != nil {
		_ = a.logger.Close()
	}
}

func (a *App) logEndpoint(name string, request any, response any, errString string) {
	if a.logger != nil {
		a.logger.LogEndpoint(name, request, response, errString)
	}
}

// LogClientEvent пишет событие фронтенда в лог
func (a *App) LogClientEvent(name string, payload map[string]any) ApiResponse[bool] {
	if a.logger != nil {
		a.logger.LogFrontendEvent(name, payload)
	}
	return ApiResponse[bool]{Data: true, Error: ""}
}

type ApiResponse[T any] struct {
	Data  T      `json:"data"`
	Error string `json:"error"`
}

// GetConfig возвращает текущую конфигурацию
func (a *App) GetConfig() ApiResponse[*config.Config] {
	resp := ApiResponse[*config.Config]{
		Data:  a.config,
		Error: "",
	}
	a.logEndpoint("GetConfig", nil, resp, resp.Error)
	return resp
}

// UpdateConfig обновляет конфигурацию
func (a *App) UpdateConfig(newConfig *config.Config) ApiResponse[bool] {
	var resp ApiResponse[bool]
	// Сохраняем новую конфигурацию
	if err := config.SaveConfig(newConfig); err != nil {
		resp = ApiResponse[bool]{
			Data:  false,
			Error: fmt.Sprintf("%s: %v", locales.GetMessage("errors.config.save_failed"), err),
		}
		a.logEndpoint("UpdateConfig", newConfig, resp, resp.Error)
		return resp
	}

	// Обновляем конфигурацию в состоянии
	a.config = newConfig

	resp = ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
	a.logEndpoint("UpdateConfig", newConfig, resp, resp.Error)
	return resp
}

// GetConfigPath возвращает путь к файлу конфигурации
func (a *App) GetConfigPath() ApiResponse[string] {
	path, err := config.GetConfigPath()
	if err != nil {
		resp := ApiResponse[string]{
			Data:  "",
			Error: fmt.Sprintf("%s: %v", locales.GetMessage("errors.config.path_failed"), err),
		}
		a.logEndpoint("GetConfigPath", nil, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[string]{
		Data:  path,
		Error: "",
	}
	a.logEndpoint("GetConfigPath", nil, resp, resp.Error)
	return resp
}

// CreateDirection создает новое направление
func (a *App) CreateDirection(req storage.CreateDirectionRequest) ApiResponse[*storage.Direction] {
	direction, err := a.storage.Directions.Create(req)
	if err != nil {
		resp := ApiResponse[*storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("CreateDirection", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Direction]{
		Data:  direction,
		Error: "",
	}
	a.logEndpoint("CreateDirection", req, resp, resp.Error)
	return resp
}

// GetDirections получает все направления
func (a *App) GetDirections() ApiResponse[[]storage.Direction] {
	directions, err := a.storage.Directions.GetAll()
	if err != nil {
		resp := ApiResponse[[]storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("GetDirections", nil, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[[]storage.Direction]{
		Data:  directions,
		Error: "",
	}
	a.logEndpoint("GetDirections", nil, resp, resp.Error)
	return resp
}

// GetDirectionsByArchived получает направления по признаку архивации (по умолчанию неархивные)
func (a *App) GetDirectionsByArchived(isArchived bool) ApiResponse[[]storage.Direction] {
	directions, err := a.storage.Directions.GetAllByArchived(isArchived)
	if err != nil {
		resp := ApiResponse[[]storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("GetDirectionsByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Direction]{Data: directions, Error: ""}
	a.logEndpoint("GetDirectionsByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
	return resp
}

// UpdateDirection обновляет направление
func (a *App) UpdateDirection(req storage.UpdateDirectionRequest) ApiResponse[*storage.Direction] {
	direction, err := a.storage.Directions.Update(req)
	if err != nil {
		resp := ApiResponse[*storage.Direction]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("UpdateDirection", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Direction]{
		Data:  direction,
		Error: "",
	}
	a.logEndpoint("UpdateDirection", req, resp, resp.Error)
	return resp
}

// DeleteDirection удаляет направление
func (a *App) DeleteDirection(id int64) ApiResponse[bool] {
	err := a.storage.Directions.Delete(id)
	if err != nil {
		resp := ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
		a.logEndpoint("DeleteDirection", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
	a.logEndpoint("DeleteDirection", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// RestoreDirection восстанавливает направление
func (a *App) RestoreDirection(id int64) ApiResponse[bool] {
	if err := a.storage.Directions.Restore(id); err != nil {
		resp := ApiResponse[bool]{Data: false, Error: err.Error()}
		a.logEndpoint("RestoreDirection", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[bool]{Data: true, Error: ""}
	a.logEndpoint("RestoreDirection", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// CreateFaculty создает новый факультет
func (a *App) CreateFaculty(req storage.CreateFacultyRequest) ApiResponse[*storage.Faculty] {
	faculty, err := a.storage.Faculties.Create(req)
	if err != nil {
		resp := ApiResponse[*storage.Faculty]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("CreateFaculty", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Faculty]{
		Data:  faculty,
		Error: "",
	}
	a.logEndpoint("CreateFaculty", req, resp, resp.Error)
	return resp
}

// GetFaculties получает все факультеты
func (a *App) GetFaculties() ApiResponse[[]storage.Faculty] {
	faculties, err := a.storage.Faculties.GetAll()
	if err != nil {
		resp := ApiResponse[[]storage.Faculty]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("GetFaculties", nil, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[[]storage.Faculty]{
		Data:  faculties,
		Error: "",
	}
	a.logEndpoint("GetFaculties", nil, resp, resp.Error)
	return resp
}

// GetFacultiesByArchived получает факультеты по признаку архивации
func (a *App) GetFacultiesByArchived(isArchived bool) ApiResponse[[]storage.Faculty] {
	faculties, err := a.storage.Faculties.GetAllByArchived(isArchived)
	if err != nil {
		resp := ApiResponse[[]storage.Faculty]{Data: nil, Error: err.Error()}
		a.logEndpoint("GetFacultiesByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Faculty]{Data: faculties, Error: ""}
	a.logEndpoint("GetFacultiesByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
	return resp
}

// UpdateFaculty обновляет факультет
func (a *App) UpdateFaculty(req storage.UpdateFacultyRequest) ApiResponse[*storage.Faculty] {
	faculty, err := a.storage.Faculties.Update(req)
	if err != nil {
		resp := ApiResponse[*storage.Faculty]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("UpdateFaculty", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Faculty]{
		Data:  faculty,
		Error: "",
	}
	a.logEndpoint("UpdateFaculty", req, resp, resp.Error)
	return resp
}

// DeleteFaculty удаляет факультет
func (a *App) DeleteFaculty(id int64) ApiResponse[bool] {
	err := a.storage.Faculties.Delete(id)
	if err != nil {
		resp := ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
		a.logEndpoint("DeleteFaculty", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
	a.logEndpoint("DeleteFaculty", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// RestoreFaculty восстанавливает факультет
func (a *App) RestoreFaculty(id int64) ApiResponse[bool] {
	if err := a.storage.Faculties.Restore(id); err != nil {
		resp := ApiResponse[bool]{Data: false, Error: err.Error()}
		a.logEndpoint("RestoreFaculty", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[bool]{Data: true, Error: ""}
	a.logEndpoint("RestoreFaculty", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// CreateTeacher создает нового преподавателя
func (a *App) CreateTeacher(req storage.CreateTeacherRequest) ApiResponse[*storage.Teacher] {
	teacher, err := a.storage.Teachers.Create(req)
	if err != nil {
		resp := ApiResponse[*storage.Teacher]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("CreateTeacher", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Teacher]{
		Data:  teacher,
		Error: "",
	}
	a.logEndpoint("CreateTeacher", req, resp, resp.Error)
	return resp
}

// GetTeachers получает всех преподавателей
func (a *App) GetTeachers() ApiResponse[[]storage.Teacher] {
	teachers, err := a.storage.Teachers.GetAll()
	if err != nil {
		resp := ApiResponse[[]storage.Teacher]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("GetTeachers", nil, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[[]storage.Teacher]{
		Data:  teachers,
		Error: "",
	}
	a.logEndpoint("GetTeachers", nil, resp, resp.Error)
	return resp
}

// GetTeachersByArchived получает преподавателей по признаку архивации
func (a *App) GetTeachersByArchived(isArchived bool) ApiResponse[[]storage.Teacher] {
	teachers, err := a.storage.Teachers.GetAllByArchived(isArchived)
	if err != nil {
		resp := ApiResponse[[]storage.Teacher]{Data: nil, Error: err.Error()}
		a.logEndpoint("GetTeachersByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Teacher]{Data: teachers, Error: ""}
	a.logEndpoint("GetTeachersByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
	return resp
}

// UpdateTeacher обновляет преподавателя
func (a *App) UpdateTeacher(req storage.UpdateTeacherRequest) ApiResponse[*storage.Teacher] {
	teacher, err := a.storage.Teachers.Update(req)
	if err != nil {
		resp := ApiResponse[*storage.Teacher]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("UpdateTeacher", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Teacher]{
		Data:  teacher,
		Error: "",
	}
	a.logEndpoint("UpdateTeacher", req, resp, resp.Error)
	return resp
}

// DeleteTeacher удаляет преподавателя
func (a *App) DeleteTeacher(id int64) ApiResponse[bool] {
	err := a.storage.Teachers.Delete(id)
	if err != nil {
		resp := ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
		a.logEndpoint("DeleteTeacher", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
	a.logEndpoint("DeleteTeacher", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// RestoreTeacher восстанавливает преподавателя
func (a *App) RestoreTeacher(id int64) ApiResponse[bool] {
	if err := a.storage.Teachers.Restore(id); err != nil {
		resp := ApiResponse[bool]{Data: false, Error: err.Error()}
		a.logEndpoint("RestoreTeacher", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[bool]{Data: true, Error: ""}
	a.logEndpoint("RestoreTeacher", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// CreateLesson создает новое занятие
func (a *App) CreateLesson(req storage.CreateLessonRequest) ApiResponse[*storage.Lesson] {
	lesson, err := a.storage.Lessons.Create(req)
	if err != nil {
		resp := ApiResponse[*storage.Lesson]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("CreateLesson", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Lesson]{
		Data:  lesson,
		Error: "",
	}
	a.logEndpoint("CreateLesson", req, resp, resp.Error)
	return resp
}

// GetLessons получает все занятия
func (a *App) GetLessons() ApiResponse[[]storage.Lesson] {
	lessons, err := a.storage.Lessons.GetAll()
	if err != nil {
		resp := ApiResponse[[]storage.Lesson]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("GetLessons", nil, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[[]storage.Lesson]{
		Data:  lessons,
		Error: "",
	}
	a.logEndpoint("GetLessons", nil, resp, resp.Error)
	return resp
}

// UpdateLesson обновляет занятие
func (a *App) UpdateLesson(req storage.UpdateLessonRequest) ApiResponse[*storage.Lesson] {
	lesson, err := a.storage.Lessons.Update(req)
	if err != nil {
		resp := ApiResponse[*storage.Lesson]{
			Data:  nil,
			Error: err.Error(),
		}
		a.logEndpoint("UpdateLesson", req, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[*storage.Lesson]{
		Data:  lesson,
		Error: "",
	}
	a.logEndpoint("UpdateLesson", req, resp, resp.Error)
	return resp
}

// DeleteLesson удаляет занятие
func (a *App) DeleteLesson(id int64) ApiResponse[bool] {
	err := a.storage.Lessons.Delete(id)
	if err != nil {
		resp := ApiResponse[bool]{
			Data:  false,
			Error: err.Error(),
		}
		a.logEndpoint("DeleteLesson", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}

	resp := ApiResponse[bool]{
		Data:  true,
		Error: "",
	}
	a.logEndpoint("DeleteLesson", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// GetSemesters получает все семестры
func (a *App) GetSemesters() ApiResponse[[]storage.Semester] {
	semesters, err := a.storage.Semesters.GetAll()
	if err != nil {
		resp := ApiResponse[[]storage.Semester]{Data: nil, Error: err.Error()}
		a.logEndpoint("GetSemesters", nil, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Semester]{Data: semesters, Error: ""}
	a.logEndpoint("GetSemesters", nil, resp, resp.Error)
	return resp
}

// GetSemestersByArchived получает семестры по признаку архивации
func (a *App) GetSemestersByArchived(isArchived bool) ApiResponse[[]storage.Semester] {
	semesters, err := a.storage.Semesters.GetAllByArchived(isArchived)
	if err != nil {
		resp := ApiResponse[[]storage.Semester]{Data: nil, Error: err.Error()}
		a.logEndpoint("GetSemestersByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Semester]{Data: semesters, Error: ""}
	a.logEndpoint("GetSemestersByArchived", map[string]any{"isArchived": isArchived}, resp, resp.Error)
	return resp
}

// CreateSemester создает новый семестр
func (a *App) CreateSemester(req storage.CreateSemesterRequest) ApiResponse[*storage.Semester] {
	semester, err := a.storage.Semesters.Create(req)
	if err != nil {
		resp := ApiResponse[*storage.Semester]{Data: nil, Error: err.Error()}
		a.logEndpoint("CreateSemester", req, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[*storage.Semester]{Data: semester, Error: ""}
	a.logEndpoint("CreateSemester", req, resp, resp.Error)
	return resp
}

// UpdateSemester обновляет семестр
func (a *App) UpdateSemester(req storage.UpdateSemesterRequest) ApiResponse[*storage.Semester] {
	semester, err := a.storage.Semesters.Update(req)
	if err != nil {
		resp := ApiResponse[*storage.Semester]{Data: nil, Error: err.Error()}
		a.logEndpoint("UpdateSemester", req, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[*storage.Semester]{Data: semester, Error: ""}
	a.logEndpoint("UpdateSemester", req, resp, resp.Error)
	return resp
}

// DeleteSemester мягко удаляет семестр
func (a *App) DeleteSemester(id int64) ApiResponse[bool] {
	if err := a.storage.Semesters.Delete(id); err != nil {
		resp := ApiResponse[bool]{Data: false, Error: err.Error()}
		a.logEndpoint("DeleteSemester", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[bool]{Data: true, Error: ""}
	a.logEndpoint("DeleteSemester", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// RestoreSemester восстанавливает семестр
func (a *App) RestoreSemester(id int64) ApiResponse[bool] {
	if err := a.storage.Semesters.Restore(id); err != nil {
		resp := ApiResponse[bool]{Data: false, Error: err.Error()}
		a.logEndpoint("RestoreSemester", map[string]any{"id": id}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[bool]{Data: true, Error: ""}
	a.logEndpoint("RestoreSemester", map[string]any{"id": id}, resp, resp.Error)
	return resp
}

// GetSemesterTeachers получает всех преподавателей семестра
func (a *App) GetSemesterTeachers(semesterID int64) ApiResponse[[]storage.Teacher] {
	teachers, err := a.storage.Semesters.GetSemesterTeachers(semesterID)
	if err != nil {
		resp := ApiResponse[[]storage.Teacher]{Data: nil, Error: err.Error()}
		a.logEndpoint("GetSemesterTeachers", map[string]any{"semesterID": semesterID}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Teacher]{Data: teachers, Error: ""}
	a.logEndpoint("GetSemesterTeachers", map[string]any{"semesterID": semesterID}, resp, resp.Error)
	return resp
}

// GetAllTeachersForSemester получает всех преподавателей доступных для семестра (включая статус привязки)
func (a *App) GetAllTeachersForSemester(semesterID int64) ApiResponse[[]storage.Teacher] {
	teachers, err := a.storage.Semesters.GetAllTeachersForSemester(semesterID)
	if err != nil {
		resp := ApiResponse[[]storage.Teacher]{Data: nil, Error: err.Error()}
		a.logEndpoint("GetAllTeachersForSemester", map[string]any{"semesterID": semesterID}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Teacher]{Data: teachers, Error: ""}
	a.logEndpoint("GetAllTeachersForSemester", map[string]any{"semesterID": semesterID}, resp, resp.Error)
	return resp
}

// BindTeacherToSemester привязывает преподавателя к семестру
func (a *App) BindTeacherToSemester(req storage.BindTeacherToSemesterRequest) ApiResponse[bool] {
	if err := a.storage.Semesters.BindTeacher(req); err != nil {
		resp := ApiResponse[bool]{Data: false, Error: err.Error()}
		a.logEndpoint("BindTeacherToSemester", req, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[bool]{Data: true, Error: ""}
	a.logEndpoint("BindTeacherToSemester", req, resp, resp.Error)
	return resp
}

// UnbindTeacherFromSemester отвязывает преподавателя от семестра
func (a *App) UnbindTeacherFromSemester(req storage.UnbindTeacherFromSemesterRequest) ApiResponse[bool] {
	if err := a.storage.Semesters.UnbindTeacher(req); err != nil {
		resp := ApiResponse[bool]{Data: false, Error: err.Error()}
		a.logEndpoint("UnbindTeacherFromSemester", req, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[bool]{Data: true, Error: ""}
	a.logEndpoint("UnbindTeacherFromSemester", req, resp, resp.Error)
	return resp
}

// GetLessonsBySemester получает все занятия семестра
func (a *App) GetLessonsBySemester(semesterID int64) ApiResponse[[]storage.Lesson] {
	lessons, err := a.storage.Lessons.GetBySemesterID(semesterID)
	if err != nil {
		resp := ApiResponse[[]storage.Lesson]{Data: nil, Error: err.Error()}
		a.logEndpoint("GetLessonsBySemester", map[string]any{"semesterID": semesterID}, resp, resp.Error)
		return resp
	}
	resp := ApiResponse[[]storage.Lesson]{Data: lessons, Error: ""}
	a.logEndpoint("GetLessonsBySemester", map[string]any{"semesterID": semesterID}, resp, resp.Error)
	return resp
}
