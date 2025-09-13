package app_services

import (
	"fmt"
	"sort"
	"strings"
	"time"

	"tulsu-pe-timetable/backend/locales"
	"tulsu-pe-timetable/backend/storage"

	"github.com/wailsapp/wails/v2/pkg/runtime"
	"github.com/xuri/excelize/v2"
)

// ExportScheduleXLSX формирует XLSX-файл расписания выбранного семестра и
// сохраняет его в папку exports рядом с базой данных. Возвращает полный путь к файлу.
func (a *App) ExportScheduleXLSX(semesterID int64) ApiResponse[string] {
	// Получаем семестр
	semester, err := a.storage.Semesters.GetByID(semesterID)
	if err != nil {
		return ApiResponse[string]{Data: "", Error: err.Error()}
	}

	// Получаем данные: занятия и преподаватели семестра
	lessons, err := a.storage.Lessons.GetBySemesterID(semesterID)
	if err != nil {
		return ApiResponse[string]{Data: "", Error: err.Error()}
	}
	teachers, err := a.storage.Semesters.GetSemesterTeachers(semesterID)
	if err != nil {
		return ApiResponse[string]{Data: "", Error: err.Error()}
	}

	// Собираем список направлений, встречающихся в занятиях
	dirOrder := make([]int64, 0)
	dirIDToName := make(map[int64]string)
	seen := make(map[int64]bool)
	for _, l := range lessons {
		if !seen[l.DirectionID] {
			seen[l.DirectionID] = true
			dirOrder = append(dirOrder, l.DirectionID)
			dirIDToName[l.DirectionID] = l.DirectionName
		}
	}
	// Стабильная сортировка по имени направления
	sort.Slice(dirOrder, func(i, j int) bool {
		return strings.ToLower(dirIDToName[dirOrder[i]]) < strings.ToLower(dirIDToName[dirOrder[j]])
	})

	// Готовим агрегаты по преподавателям (кол-во и сумма ставок по направлениям)
	teacherCountByDir := make(map[int64]int)
	rateSumByDir := make(map[int64]float64)
	for _, t := range teachers {
		teacherCountByDir[t.DirectionID]++
		rateSumByDir[t.DirectionID] += t.Rate
		// Если в семестре нет занятий этого направления, всё равно покажем колонку
		if _, ok := dirIDToName[t.DirectionID]; !ok {
			dirIDToName[t.DirectionID] = t.DirectionName
			dirOrder = append(dirOrder, t.DirectionID)
		}
	}
	// Пересортируем после возможного пополнения
	sort.SliceStable(dirOrder, func(i, j int) bool {
		return strings.ToLower(dirIDToName[dirOrder[i]]) < strings.ToLower(dirIDToName[dirOrder[j]])
	})

	// Карта day->уникальные слоты, отсортированные
	dayToSlots := map[int][]string{}
	type key struct {
		day  int
		slot string
	}
	unique := map[key]bool{}
	normalizeTime := func(v string) string {
		// Обрезаем секунды, если есть
		parts := strings.Split(v, ":")
		if len(parts) >= 2 {
			return fmt.Sprintf("%02s:%02s", parts[0], parts[1])
		}
		return v
	}
	for _, l := range lessons {
		start := normalizeTime(l.StartTime)
		end := normalizeTime(l.EndTime)
		slot := start + "-" + end
		k := key{day: l.DayOfWeek, slot: slot}
		if !unique[k] {
			unique[k] = true
			dayToSlots[l.DayOfWeek] = append(dayToSlots[l.DayOfWeek], slot)
		}
	}
	// Сортировка слотов по времени
	parseMinutes := func(slot string) int {
		parts := strings.Split(slot, "-")
		if len(parts) != 2 {
			return 0
		}
		p := strings.Split(parts[0], ":")
		if len(p) < 2 {
			return 0
		}
		return atoiSafe(p[0])*60 + atoiSafe(p[1])
	}
	for d := 1; d <= 7; d++ {
		slots := dayToSlots[d]
		sort.Slice(slots, func(i, j int) bool { return parseMinutes(slots[i]) < parseMinutes(slots[j]) })
		dayToSlots[d] = slots
	}

	// Создаем книгу
	f := excelize.NewFile()
	const defaultSheet = "Sheet1"
	const sheetName = "Расписание"
	_ = f.SetSheetName(defaultSheet, sheetName)

	// Параметры страницы: A4, альбомная ориентация, подгон по ширине/высоте
	orient := "landscape"
	size := 9 // A4
	fitW := 1
	fitH := 1
	_ = f.SetPageLayout(sheetName, &excelize.PageLayoutOptions{
		Orientation: &orient,
		Size:        &size,
		FitToWidth:  &fitW,
		FitToHeight: &fitH,
	})
	top := 0.5
	bottom := 0.5
	leftM := 0.25
	rightM := 0.25
	headerM := 0.2
	footerM := 0.2
	_ = f.SetPageMargins(sheetName, &excelize.PageLayoutMarginsOptions{
		Top:    &top,
		Bottom: &bottom,
		Left:   &leftM,
		Right:  &rightM,
		Header: &headerM,
		Footer: &footerM,
	})

	// Колонки: День | Время | Факультеты | <направления...>
	colDay := 1
	colTime := 2
	colFac := 3
	colFirstDir := 4

	// Стили для таблицы (Times New Roman, тонкие границы)
	headerStyle, _ := f.NewStyle(&excelize.Style{
		Font:      &excelize.Font{Family: "Times New Roman", Bold: true, Size: 11},
		Alignment: &excelize.Alignment{Horizontal: "center", Vertical: "center", WrapText: true},
		Border: []excelize.Border{
			{Type: "left", Color: "#000000", Style: 1},
			{Type: "right", Color: "#000000", Style: 1},
			{Type: "top", Color: "#000000", Style: 1},
			{Type: "bottom", Color: "#000000", Style: 1},
		},
	})
	gridStyle, _ := f.NewStyle(&excelize.Style{
		Font:      &excelize.Font{Family: "Times New Roman", Size: 11},
		Alignment: &excelize.Alignment{Vertical: "top", WrapText: true},
		Border: []excelize.Border{
			{Type: "left", Color: "#000000", Style: 1},
			{Type: "right", Color: "#000000", Style: 1},
			{Type: "top", Color: "#000000", Style: 1},
			{Type: "bottom", Color: "#000000", Style: 1},
		},
	})
	centerGridStyle, _ := f.NewStyle(&excelize.Style{
		Font:      &excelize.Font{Family: "Times New Roman", Size: 11},
		Alignment: &excelize.Alignment{Horizontal: "center", Vertical: "center", WrapText: true},
		Border: []excelize.Border{
			{Type: "left", Color: "#000000", Style: 1},
			{Type: "right", Color: "#000000", Style: 1},
			{Type: "top", Color: "#000000", Style: 1},
			{Type: "bottom", Color: "#000000", Style: 1},
		},
	})
	rateNumberStyle, _ := f.NewStyle(&excelize.Style{
		Font:      &excelize.Font{Family: "Times New Roman", Size: 11},
		NumFmt:    2, // 0.00
		Alignment: &excelize.Alignment{Horizontal: "center", Vertical: "center"},
		Border: []excelize.Border{
			{Type: "left", Color: "#000000", Style: 1},
			{Type: "right", Color: "#000000", Style: 1},
			{Type: "top", Color: "#000000", Style: 1},
			{Type: "bottom", Color: "#000000", Style: 1},
		},
	})
	titleStyle, _ := f.NewStyle(&excelize.Style{
		Font:      &excelize.Font{Family: "Times New Roman", Bold: true, Size: 14},
		Alignment: &excelize.Alignment{Horizontal: "center", Vertical: "center"},
	})
	labelMergedStyle, _ := f.NewStyle(&excelize.Style{
		Font:      &excelize.Font{Family: "Times New Roman", Size: 11},
		Alignment: &excelize.Alignment{Horizontal: "left", Vertical: "center", WrapText: true},
		Border: []excelize.Border{
			{Type: "left", Color: "#000000", Style: 1},
			{Type: "right", Color: "#000000", Style: 1},
			{Type: "top", Color: "#000000", Style: 1},
			{Type: "bottom", Color: "#000000", Style: 1},
		},
	})

	row := 1
	// Титульная строка
	lastHeaderCol := colFirstDir + len(dirOrder) - 1
	if lastHeaderCol < colFac {
		lastHeaderCol = colFac
	}
	titleStart := mustCell(colDay, row)
	titleEnd, _ := excelize.CoordinatesToCellName(lastHeaderCol, row)
	_ = f.MergeCell(sheetName, titleStart, titleEnd)
	_ = f.SetCellValue(sheetName, titleStart, fmt.Sprintf("Расписание учебных занятий по физической культуре на %s", strings.TrimSpace(semester.Name)))
	_ = f.SetRowHeight(sheetName, row, 24)
	_ = f.SetCellStyle(sheetName, titleStart, titleEnd, titleStyle)

	// Шапка таблицы: только направления (левые колонки пустые)
	row++
	for i, dirID := range dirOrder {
		_ = f.SetCellValue(sheetName, mustCell(colFirstDir+i, row), strings.TrimSpace(dirIDToName[dirID]))
	}
	headerStart := mustCell(colDay, row)
	headerEnd, _ := excelize.CoordinatesToCellName(lastHeaderCol, row)
	_ = f.SetCellStyle(sheetName, headerStart, headerEnd, headerStyle)
	_ = f.SetRowHeight(sheetName, row, 24)

	// Строка "Количество преподавателей"
	row++
	// Объединяем A:C для текста
	_ = f.MergeCell(sheetName, mustCell(colDay, row), mustCell(colFac, row))
	_ = f.SetCellValue(sheetName, mustCell(colDay, row), "Количество преподавателей")
	for i, dirID := range dirOrder {
		_ = f.SetCellValue(sheetName, mustCell(colFirstDir+i, row), teacherCountByDir[dirID])
	}
	_ = f.SetCellStyle(sheetName, mustCell(colDay, row), mustCell(colFac, row), labelMergedStyle)
	_ = f.SetCellStyle(sheetName, mustCell(colFirstDir, row), headerEnd, centerGridStyle)

	// Строка "Величина ставки"
	row++
	// Объединяем A:C для текста
	_ = f.MergeCell(sheetName, mustCell(colDay, row), mustCell(colFac, row))
	_ = f.SetCellValue(sheetName, mustCell(colDay, row), "Величина ставки")
	for i, dirID := range dirOrder {
		_ = f.SetCellValue(sheetName, mustCell(colFirstDir+i, row), rateSumByDir[dirID])
	}
	_ = f.SetCellStyle(sheetName, mustCell(colDay, row), mustCell(colFac, row), labelMergedStyle)
	if len(dirOrder) > 0 {
		rateStart := mustCell(colFirstDir, row)
		rateEnd, _ := excelize.CoordinatesToCellName(lastHeaderCol, row)
		_ = f.SetCellStyle(sheetName, rateStart, rateEnd, rateNumberStyle)
	}

	// Строка заголовков для основной сетки: Дн | Время | Факультет
	row++
	_ = f.SetCellValue(sheetName, mustCell(colDay, row), "Дн")
	_ = f.SetCellValue(sheetName, mustCell(colTime, row), "Время")
	_ = f.SetCellValue(sheetName, mustCell(colFac, row), "Факультет")
	_ = f.SetCellStyle(sheetName, mustCell(colDay, row), mustCell(colFac, row), headerStyle)
	// выровняем верх границы направлений в этой строке
	if len(dirOrder) > 0 {
		gridHeaderEnd, _ := excelize.CoordinatesToCellName(lastHeaderCol, row)
		_ = f.SetCellStyle(sheetName, mustCell(colFirstDir, row), gridHeaderEnd, headerStyle)
	}

	// Основное тело: по дням и слотам
	dayName := func(d int) string {
		switch d {
		case 1:
			return "Пн"
		case 2:
			return "Вт"
		case 3:
			return "Ср"
		case 4:
			return "Чт"
		case 5:
			return "Пт"
		case 6:
			return "Сб"
		case 7:
			return "Вс"
		default:
			return fmt.Sprintf("%d", d)
		}
	}

	// Индексируем занятия для быстрого доступа
	// map[day][slot][directionID] -> []Lesson
	byKey := map[int]map[string]map[int64][]storage.Lesson{}
	for _, l := range lessons {
		start := normalizeTime(l.StartTime)
		end := normalizeTime(l.EndTime)
		slot := start + "-" + end
		if _, ok := byKey[l.DayOfWeek]; !ok {
			byKey[l.DayOfWeek] = map[string]map[int64][]storage.Lesson{}
		}
		if _, ok := byKey[l.DayOfWeek][slot]; !ok {
			byKey[l.DayOfWeek][slot] = map[int64][]storage.Lesson{}
		}
		byKey[l.DayOfWeek][slot][l.DirectionID] = append(byKey[l.DayOfWeek][slot][l.DirectionID], l)
	}

	for d := 1; d <= 7; d++ {
		slots := dayToSlots[d]
		if len(slots) == 0 {
			continue
		}
		dayStartRow := row + 1
		for _, slot := range slots {
			row++
			_ = f.SetCellValue(sheetName, mustCell(colTime, row), slot)
			// Колонка факультетов — агрегат по всем направлениям в этом слоте
			facSet := map[string]bool{}
			for _, dirID := range dirOrder {
				for _, l := range byKey[d][slot][dirID] {
					for _, name := range l.FacultyNames {
						if name != "" {
							facSet[name] = true
						}
					}
				}
			}
			facList := make([]string, 0, len(facSet))
			for name := range facSet {
				facList = append(facList, name)
			}
			sort.Strings(facList)
			_ = f.SetCellValue(sheetName, mustCell(colFac, row), strings.Join(facList, ", "))

			// Заполняем ячейки по направлениям
			for i, dirID := range dirOrder {
				items := byKey[d][slot][dirID]
				if len(items) == 0 {
					continue
				}
				var cellParts []string
				for _, l := range items {
					fac := strings.Join(l.FacultyNames, ", ")
					teachers := strings.Join(l.TeacherNames, ", ")
					val := strings.TrimSpace(strings.Join([]string{fac, teachers}, "\n"))
					if val != "" {
						cellParts = append(cellParts, val)
					}
				}
				_ = f.SetCellValue(sheetName, mustCell(colFirstDir+i, row), strings.Join(cellParts, "\n—\n"))
			}

			// применим стиль сетки к строке
			rowStart := mustCell(colDay, row)
			rowEnd, _ := excelize.CoordinatesToCellName(colFirstDir+len(dirOrder)-1, row)
			if (colFirstDir + len(dirOrder) - 1) < colFac {
				rowEnd = mustCell(colFac, row)
			}
			_ = f.SetCellStyle(sheetName, rowStart, rowEnd, gridStyle)
			_ = f.SetRowHeight(sheetName, row, 18)
		}
		// объединяем ячейку дня
		dayTop := mustCell(colDay, dayStartRow)
		dayBottom, _ := excelize.CoordinatesToCellName(colDay, row)
		_ = f.MergeCell(sheetName, dayTop, dayBottom)
		_ = f.SetCellValue(sheetName, dayTop, dayName(d))
		_ = f.SetCellStyle(sheetName, dayTop, dayBottom, centerGridStyle)
	}

	// Ширины колонок
	_ = f.SetColWidth(sheetName, "A", "A", 4)
	_ = f.SetColWidth(sheetName, "B", "B", 12)
	_ = f.SetColWidth(sheetName, "C", "C", 28)
	if len(dirOrder) > 0 {
		firstColName, _ := excelize.ColumnNumberToName(colFirstDir)
		lastColName, _ := excelize.ColumnNumberToName(colFirstDir + len(dirOrder) - 1)
		_ = f.SetColWidth(sheetName, firstColName, lastColName, 18)
	}

	// Область печати: вся таблица с заголовком и строкой места проведения
	lastColName, _ := excelize.ColumnNumberToName(lastHeaderCol)
	_ = f.SetDefinedName(&excelize.DefinedName{
		Name:     "_xlnm.Print_Area",
		RefersTo: fmt.Sprintf("='%s'!$A$1:$%s$%d", sheetName, lastColName, row),
		Scope:    sheetName,
	})

	// Диалог сохранения файла
	defaultName := fmt.Sprintf("schedule_%s_%s.xlsx", sanitizeFileName(semester.Name), time.Now().Format("20060102_150405"))
	savePath, err := runtime.SaveFileDialog(a.ctx, runtime.SaveDialogOptions{
		Title:           "Сохранить XLSX",
		DefaultFilename: defaultName,
		Filters:         []runtime.FileFilter{{DisplayName: "Excel (*.xlsx)", Pattern: "*.xlsx"}},
	})
	if err != nil {
		return ApiResponse[string]{Data: "", Error: fmt.Sprintf("%s: %v", locales.GetMessage("errors.export.dialog_failed"), err)}
	}
	if strings.TrimSpace(savePath) == "" {
		// пользователь отменил
		return ApiResponse[string]{Data: "", Error: ""}
	}
	if err := f.SaveAs(savePath); err != nil {
		return ApiResponse[string]{Data: "", Error: fmt.Sprintf("%s: %v", locales.GetMessage("errors.export.save_failed"), err)}
	}

	return ApiResponse[string]{Data: savePath, Error: ""}
}

func mustCell(col, row int) string {
	name, _ := excelize.CoordinatesToCellName(col, row)
	return name
}

func sanitizeFileName(s string) string {
	s = strings.TrimSpace(s)
	s = strings.ReplaceAll(s, " ", "_")
	// Уберем небезопасные символы
	replacer := strings.NewReplacer(
		"/", "-",
		"\\", "-",
		":", "-",
		"*", "-",
		"?", "-",
		"\"", "-",
		"<", "-",
		">", "-",
		"|", "-",
	)
	return replacer.Replace(s)
}

func atoiSafe(s string) int {
	var n int
	for _, ch := range s {
		if ch < '0' || ch > '9' {
			break
		}
		n = n*10 + int(ch-'0')
	}
	return n
}
