package locales

import (
	"encoding/json"
	"fmt"
	"os"
)

var messages map[string]interface{}

// LoadMessages загружает сообщения из файла локализации
func LoadMessages() error {
	// Путь к файлу локализации относительно корня проекта
	localePath := "backend/locales/ru.json"
	
	data, err := os.ReadFile(localePath)
	if err != nil {
		return fmt.Errorf("не удалось прочитать файл локализации: %w", err)
	}
	
	if err := json.Unmarshal(data, &messages); err != nil {
		return fmt.Errorf("не удалось распарсить файл локализации: %w", err)
	}
	
	return nil
}

// GetMessage получает сообщение по ключу
func GetMessage(key string) string {
	if messages == nil {
		if err := LoadMessages(); err != nil {
			return key // Возвращаем ключ, если не удалось загрузить локализацию
		}
	}
	
	// Простая реализация получения вложенных ключей
	keys := splitKey(key)
	current := messages
	
	for _, k := range keys {
		if val, ok := current[k]; ok {
			if str, ok := val.(string); ok {
				return str
			}
			if nested, ok := val.(map[string]interface{}); ok {
				current = nested
			} else {
				break
			}
		} else {
			break
		}
	}
	
	return key // Возвращаем ключ, если сообщение не найдено
}

// splitKey разбивает ключ по точкам
func splitKey(key string) []string {
	var result []string
	var current string
	
	for _, char := range key {
		if char == '.' {
			if current != "" {
				result = append(result, current)
				current = ""
			}
		} else {
			current += string(char)
		}
	}
	
	if current != "" {
		result = append(result, current)
	}
	
	return result
}
