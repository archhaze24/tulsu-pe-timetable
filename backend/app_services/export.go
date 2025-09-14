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

	// Формируем список колонок по преподавателям (как в UI)
	type teacherHeader struct {
		ID        int64
		ShortName string
		Direction string
		Rate      float64
	}
	formatShort := func(t storage.Teacher) string {
		first := strings.TrimSpace(t.FirstName)
		middle := strings.TrimSpace(t.MiddleName)
		last := strings.TrimSpace(t.LastName)
		var parts []string
		if last != "" {
			parts = append(parts, last)
		}
		if first != "" {
			r := []rune(first)
			if len(r) > 0 {
				parts = append(parts, string(r[0])+".")
			}
		}
		if middle != "" {
			r := []rune(middle)
			if len(r) > 0 {
				parts = append(parts, string(r[0])+".")
			}
		}
		return strings.Join(parts, " ")
	}
	teacherCols := make([]teacherHeader, 0, len(teachers))
	for _, t := range teachers {
		teacherCols = append(teacherCols, teacherHeader{
			ID:        t.ID,
			ShortName: formatShort(t),
			Direction: strings.TrimSpace(t.DirectionName),
			Rate:      t.Rate,
		})
	}

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

	// Колонки: День | Время | Факультеты | <преподаватели...>
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
	titleStyle, _ := f.NewStyle(&excelize.Style{
		Font:      &excelize.Font{Family: "Times New Roman", Bold: true, Size: 14},
		Alignment: &excelize.Alignment{Horizontal: "center", Vertical: "center", WrapText: true},
		Border: []excelize.Border{
			{Type: "left", Color: "#000000", Style: 1},
			{Type: "right", Color: "#000000", Style: 1},
			{Type: "top", Color: "#000000", Style: 1},
			{Type: "bottom", Color: "#000000", Style: 1},
		},
	})

	row := 1
	// Шапка: объединяем A:C и переносим туда название; далее ФИО преподавателя (направление)
	lastHeaderCol := colFirstDir + len(teacherCols) - 1
	if lastHeaderCol < colFac {
		lastHeaderCol = colFac
	}
	_ = f.MergeCell(sheetName, mustCell(colDay, row), mustCell(colFac, row))
	_ = f.SetCellValue(sheetName, mustCell(colDay, row), fmt.Sprintf("Расписание учебных занятий по физической культуре на %s", strings.TrimSpace(semester.Name)))
	_ = f.SetRowHeight(sheetName, row, 48)
	for i, th := range teacherCols {
		header := strings.TrimSpace(th.ShortName)
		if th.Direction != "" {
			header = header + " (" + th.Direction + ")"
		}
		_ = f.SetCellValue(sheetName, mustCell(colFirstDir+i, row), header)
	}
	// Apply styles: title on A:C, header on teacher columns
	headerEnd, _ := excelize.CoordinatesToCellName(lastHeaderCol, row)
	if len(teacherCols) > 0 {
		_ = f.SetCellStyle(sheetName, mustCell(colFirstDir, row), headerEnd, headerStyle)
	}
	_ = f.SetCellStyle(sheetName, mustCell(colDay, row), mustCell(colFac, row), titleStyle)
	_ = f.SetRowHeight(sheetName, row, 48)

	// (Строка "Количество преподавателей" удалена по требованию)

	// Строка "Величина ставки"
	row++
	// Объединяем A:C для текста
	_ = f.MergeCell(sheetName, mustCell(colDay, row), mustCell(colFac, row))
	_ = f.SetCellValue(sheetName, mustCell(colDay, row), "Величина ставки")
	for i, th := range teacherCols {
		_ = f.SetCellValue(sheetName, mustCell(colFirstDir+i, row), th.Rate)
	}
	_ = f.SetCellStyle(sheetName, mustCell(colDay, row), mustCell(colFac, row), labelMergedStyle)
	if len(teacherCols) > 0 {
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
	// выровняем верх границы колонок преподавателей в этой строке
	if len(teacherCols) > 0 {
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
	// byDaySlot: map[day][slot] -> []Lesson
	byDaySlot := map[int]map[string][]storage.Lesson{}
	// byTeacher: map[day][slot][teacherID] -> bool (наличие пары у препода в слоте)
	byTeacher := map[int]map[string]map[int64]bool{}
	for _, l := range lessons {
		start := normalizeTime(l.StartTime)
		end := normalizeTime(l.EndTime)
		slot := start + "-" + end
		if _, ok := byDaySlot[l.DayOfWeek]; !ok {
			byDaySlot[l.DayOfWeek] = map[string][]storage.Lesson{}
		}
		byDaySlot[l.DayOfWeek][slot] = append(byDaySlot[l.DayOfWeek][slot], l)
		if _, ok := byTeacher[l.DayOfWeek]; !ok {
			byTeacher[l.DayOfWeek] = map[string]map[int64]bool{}
		}
		if _, ok := byTeacher[l.DayOfWeek][slot]; !ok {
			byTeacher[l.DayOfWeek][slot] = map[int64]bool{}
		}
		for _, tid := range l.TeacherIDs {
			byTeacher[l.DayOfWeek][slot][tid] = true
		}
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
			// Колонка факультетов — агрегат по всем занятиям в этом слоте
			facSet := map[string]bool{}
			for _, l := range byDaySlot[d][slot] {
				for _, name := range l.FacultyNames {
					if name != "" {
						facSet[name] = true
					}
				}
			}
			facList := make([]string, 0, len(facSet))
			for name := range facSet {
				facList = append(facList, name)
			}
			sort.Strings(facList)
			_ = f.SetCellValue(sheetName, mustCell(colFac, row), strings.Join(facList, ", "))

			// Заполняем ячейки по преподавателям: '+' если у преподавателя есть пара в этом слоте
			for i, th := range teacherCols {
				val := ""
				if byTeacher[d][slot][th.ID] {
					val = "+"
				}
				_ = f.SetCellValue(sheetName, mustCell(colFirstDir+i, row), val)
			}

			// применим стиль сетки к строке
			rowStart := mustCell(colDay, row)
			rowEnd, _ := excelize.CoordinatesToCellName(colFirstDir+len(teacherCols)-1, row)
			if (colFirstDir + len(teacherCols) - 1) < colFac {
				rowEnd = mustCell(colFac, row)
			}
			_ = f.SetCellStyle(sheetName, rowStart, rowEnd, gridStyle)
			_ = f.SetRowHeight(sheetName, row, 18)

			// выровняем плюсики по центру в колонках преподавателей
			if len(teacherCols) > 0 {
				teacherRowStart := mustCell(colFirstDir, row)
				teacherRowEnd, _ := excelize.CoordinatesToCellName(colFirstDir+len(teacherCols)-1, row)
				_ = f.SetCellStyle(sheetName, teacherRowStart, teacherRowEnd, centerGridStyle)
			}
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
	if len(teacherCols) > 0 {
		firstColName, _ := excelize.ColumnNumberToName(colFirstDir)
		lastColName, _ := excelize.ColumnNumberToName(colFirstDir + len(teacherCols) - 1)
		_ = f.SetColWidth(sheetName, firstColName, lastColName, 18)
	}

	// Область печати: вся таблица с заголовком и строкой места проведения
	lastColName, _ := excelize.ColumnNumberToName(lastHeaderCol)
	// Уточним последнюю непустую строку по фактическим значениям ячеек и, при необходимости, удалим хвост
	{
		lastNonEmptyRow := 0
		for r := row; r >= 1; r-- {
			has := false
			for c := 1; c <= lastHeaderCol; c++ {
				cell, _ := excelize.CoordinatesToCellName(c, r)
				v, _ := f.GetCellValue(sheetName, cell)
				if strings.TrimSpace(v) != "" {
					has = true
					break
				}
			}
			if has {
				lastNonEmptyRow = r
				break
			}
		}
		if lastNonEmptyRow > 0 && lastNonEmptyRow < row {
			// Удаляем все строки после последней непустой
			for toRemove := row - lastNonEmptyRow; toRemove > 0; toRemove-- {
				_ = f.RemoveRow(sheetName, lastNonEmptyRow+1)
			}
			row = lastNonEmptyRow
		}
	}
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
