package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"tulsu-pe-timetable/backend/locales"
)

// Config структура конфигурации приложения
type Config struct {
	DbPath string `json:"dbPath"`
}

// GetConfigPath возвращает путь к файлу конфигурации в зависимости от ОС
func GetConfigPath() (string, error) {
	configDir, err := os.UserConfigDir()
	if err != nil {
		return "", fmt.Errorf(locales.GetMessage("errors.config.path_failed")+": %w", err)
	}
	
	// Создаем папку для нашего приложения
	appConfigDir := filepath.Join(configDir, "tulsu-pe-timetable")
	
	// Создаем папку, если её нет
	if err := os.MkdirAll(appConfigDir, 0755); err != nil {
		return "", fmt.Errorf(locales.GetMessage("errors.config.create_dir_failed")+": %w", err)
	}
	
	return filepath.Join(appConfigDir, "config.json"), nil
}

// LoadConfig загружает конфигурацию из файла
func LoadConfig() (*Config, error) {
	configPath, err := GetConfigPath()
	if err != nil {
		return nil, err
	}
	
	// Проверяем, существует ли файл конфигурации
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		// Создаем конфигурацию по умолчанию
		return CreateDefaultConfig()
	}
	
	// Читаем существующий файл
	data, err := os.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.config.read_failed")+": %w", err)
	}
	
	var config Config
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.config.parse_failed")+": %w", err)
	}
	
	return &config, nil
}

// CreateDefaultConfig создает конфигурацию по умолчанию
func CreateDefaultConfig() (*Config, error) {
	_, err := GetConfigPath()
	if err != nil {
		return nil, err
	}
	
	// Получаем путь к папке данных пользователя
	dataDir, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.config.home_dir_failed")+": %w", err)
	}
	
	// Создаем папку для данных приложения
	appDataDir := filepath.Join(dataDir, ".tulsu-pe-timetable")
	if err := os.MkdirAll(appDataDir, 0755); err != nil {
		return nil, fmt.Errorf(locales.GetMessage("errors.config.data_dir_failed")+": %w", err)
	}
	
	// Путь к базе данных по умолчанию
	dbPath := filepath.Join(appDataDir, "timetable.db")
	
	config := &Config{
		DbPath: dbPath,
	}
	
	// Сохраняем конфигурацию
	if err := SaveConfig(config); err != nil {
		return nil, err
	}
	
	return config, nil
}

// SaveConfig сохраняет конфигурацию в файл
func SaveConfig(config *Config) error {
	configPath, err := GetConfigPath()
	if err != nil {
		return err
	}
	
	data, err := json.MarshalIndent(config, "", "  ")
	if err != nil {
		return fmt.Errorf(locales.GetMessage("errors.config.save_failed")+": %w", err)
	}
	
	if err := os.WriteFile(configPath, data, 0644); err != nil {
		return fmt.Errorf(locales.GetMessage("errors.config.save_failed")+": %w", err)
	}
	
	return nil
}
