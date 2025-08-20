package app_services

import (
	"context"
	"fmt"
	"tulsu-pe-timetable/backend/config"
	"tulsu-pe-timetable/backend/locales"
)

// App struct
type App struct {
	ctx    context.Context
	config *config.Config
}

// NewApp creates a new App application struct
func NewApp(cfg *config.Config) *App {
	return &App{
		config: cfg,
	}
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) Startup(ctx context.Context) {
	a.ctx = ctx
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
