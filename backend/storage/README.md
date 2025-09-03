# Storage Layer

Storage слой для работы с базой данных SQLite в приложении расписания ТулГУ.

## Структура

```
storage/
├── database.go      # Инициализация БД и создание таблиц
├── models.go        # Модели данных
├── directions.go    # Репозиторий направлений
├── faculties.go     # Репозиторий факультетов
├── teachers.go      # Репозиторий преподавателей
├── lessons.go       # Репозиторий занятий
├── storage.go       # Главный менеджер storage
└── README.md        # Документация
```

## Модели данных

### Direction (Направление)
- `ID` - уникальный идентификатор
- `Name` - название направления (футбол, плавание и т.д.)
- `Description` - описание направления
- `CreatedAt`, `UpdatedAt` - временные метки

### Faculty (Факультет)
- `ID` - уникальный идентификатор
- `Name` - полное название факультета
- `CreatedAt`, `UpdatedAt` - временные метки

### Teacher (Преподаватель)
- `ID` - уникальный идентификатор
- `LastName`, `FirstName`, `MiddleName` - ФИО
- `Rate` - ставка (полная ставка = 1.0, соответствует 12 занятиям в неделю)
- `CreatedAt`, `UpdatedAt` - временные метки

### Lesson (Занятие)
- `ID` - уникальный идентификатор
- `FacultyID` - ID факультета
- `DirectionID` - ID направления
- `TeacherID` - ID преподавателя (может быть null)
- `DayOfWeek` - день недели (1-7)
- `LessonNumber` - номер пары (1-8)
- `CreatedAt`, `UpdatedAt` - временные метки
- `FacultyName`, `DirectionName`, `TeacherName` - связанные данные (для JOIN запросов)

## Использование

### Инициализация

```go
// Загружаем конфигурацию
cfg, err := config.LoadConfig()
if err != nil {
    log.Fatal(err)
}

// Создаем storage
storage, err := storage.NewStorage(cfg)
if err != nil {
    log.Fatal(err)
}
defer storage.Close()
```

### Работа с направлениями

```go
// Создание
direction, err := storage.Directions.Create(storage.CreateDirectionRequest{
    Name:        "Футбол",
    Description: "Игра в футбол",
})

// Получение всех
directions, err := storage.Directions.GetAll()

// Обновление
updated, err := storage.Directions.Update(storage.UpdateDirectionRequest{
    ID:          1,
    Name:        "Футбол обновленный",
    Description: "Новое описание",
})

// Удаление
err := storage.Directions.Delete(1)
```

### Работа с факультетами

```go
// Создание
faculty, err := storage.Faculties.Create(storage.CreateFacultyRequest{
    Name:      "Факультет информационных технологий",
})

// Получение всех
faculties, err := storage.Faculties.GetAll()
```

### Работа с преподавателями

```go
// Создание
teacher, err := storage.Teachers.Create(storage.CreateTeacherRequest{
    LastName:   "Иванов",
    FirstName:  "Иван",
    MiddleName: "Иванович",
    Rate:       1.0, // полная ставка
})

// Получение количества занятий
count, err := storage.Teachers.GetLessonsCount(teacherID)
```

### Работа с занятиями

```go
// Создание
lesson, err := storage.Lessons.Create(storage.CreateLessonRequest{
    FacultyID:    1,
    DirectionID:  1,
    TeacherID:    &teacherID, // указатель, может быть nil
    DayOfWeek:    1,          // понедельник
    LessonNumber: 1,          // первая пара
})

// Получение всех с связанными данными
lessons, err := storage.Lessons.GetAll()
```

## Особенности

1. **Автоматическое создание таблиц** - при инициализации БД все необходимые таблицы создаются автоматически
2. **Внешние ключи** - настроены связи между таблицами с каскадным удалением
3. **Уникальные ограничения** - предотвращают дублирование данных
4. **Локализация ошибок** - все ошибки берутся из файла локализации
5. **Унифицированный API** - все методы возвращают одинаковую структуру ответа

## Добавление новых методов

Для добавления новых методов в репозиторий:

1. Добавьте метод в соответствующий файл репозитория
2. Добавьте сообщение об ошибке в `backend/locales/ru.json`
3. Добавьте метод в `backend/app_services/app.go` для экспорта через Wails API

Пример добавления метода:

```go
// В teachers.go
func (r *TeachersRepository) GetByRate(rate float64) ([]Teacher, error) {
    query := `SELECT * FROM teachers WHERE rate = ?`
    // реализация...
}

// В app.go
func (a *App) GetTeachersByRate(rate float64) ApiResponse[[]storage.Teacher] {
    teachers, err := a.storage.Teachers.GetByRate(rate)
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
```
