use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use chrono::Local;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};

use crate::error::{AppError, AppResult};
use crate::logger;
use crate::models::{Lesson, Semester, Teacher};

const SHEET_NAME: &str = "Расписание";
const DAY_NAMES: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

/// Имя файла по умолчанию для диалога сохранения на фронтенде.
pub fn default_file_name(semester: &Semester) -> String {
    format!(
        "schedule_{}_{}.xlsx",
        sanitize_file_name(&semester.name),
        Local::now().format("%Y%m%d_%H%M%S")
    )
}

/// Формирует XLSX по образцу Go-версии и сохраняет его по указанному пути.
/// Выбор пути делает фронтенд (диалог сохранения), здесь только запись.
pub fn write_schedule(path: &Path, semester: &Semester, lessons: &[Lesson], teachers: &[Teacher]) -> AppResult<()> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    write_sheet(sheet, semester, lessons, teachers).map_err(|e| AppError::with_cause("errors.export.save_failed", e))?;
    workbook
        .save(path)
        .map_err(|e| AppError::with_cause("errors.export.save_failed", e))?;
    logger::info("export", format!("XLSX сохранён: {}", path.display()));
    Ok(())
}

fn base_border() -> Format {
    Format::new()
        .set_font_name("Times New Roman")
        .set_font_size(11)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::Black)
}

fn write_sheet(
    sheet: &mut Worksheet,
    semester: &Semester,
    lessons: &[Lesson],
    teachers: &[Teacher],
) -> Result<(), XlsxError> {
    sheet.set_name(SHEET_NAME)?;
    sheet.set_landscape();
    sheet.set_paper_size(9); // A4
    sheet.set_print_scale(100);
    sheet.set_margins(0.25, 0.25, 0.5, 0.5, 0.2, 0.2);

    let title = base_border().set_bold().set_font_size(14).set_align(FormatAlign::Center).set_text_wrap();
    let header = base_border().set_bold().set_align(FormatAlign::Center).set_text_wrap();
    let cell = base_border().set_align(FormatAlign::Top).set_text_wrap();
    let centered = base_border().set_align(FormatAlign::Center);
    let rate = base_border().set_num_format("0.00").set_align(FormatAlign::Center);
    let label = base_border().set_align(FormatAlign::Left).set_text_wrap();

    // Колонки: A — день, B — время, C — факультеты, D… — преподаватели
    const COL_DAY: u16 = 0;
    const COL_TIME: u16 = 1;
    const COL_FAC: u16 = 2;
    const COL_FIRST_TEACHER: u16 = 3;

    sheet.set_column_width(COL_DAY, 4)?;
    sheet.set_column_width(COL_TIME, 12)?;
    sheet.set_column_width(COL_FAC, 28)?;
    for offset in 0..teachers.len() as u16 {
        sheet.set_column_width(COL_FIRST_TEACHER + offset, 18)?;
    }

    // Шапка: название семестра и ФИО преподавателей с направлением
    sheet.set_row_height(0, 48)?;
    sheet.merge_range(0, COL_DAY, 0, COL_FAC, &format!("Расписание учебных занятий по физической культуре на {}", semester.name.trim()), &title)?;
    for (i, teacher) in teachers.iter().enumerate() {
        let mut text = short_name(teacher);
        if !teacher.direction_name.trim().is_empty() {
            text = format!("{} ({})", text, teacher.direction_name.trim());
        }
        sheet.write_string_with_format(0, COL_FIRST_TEACHER + i as u16, &text, &header)?;
    }

    // Строка ставок
    sheet.set_row_height(1, 18)?;
    sheet.merge_range(1, COL_DAY, 1, COL_FAC, "Величина ставки", &label)?;
    for (i, teacher) in teachers.iter().enumerate() {
        sheet.write_number_with_format(1, COL_FIRST_TEACHER + i as u16, teacher.rate, &rate)?;
    }

    // Строка заголовков сетки
    sheet.write_string_with_format(2, COL_DAY, "Дн", &header)?;
    sheet.write_string_with_format(2, COL_TIME, "Время", &header)?;
    sheet.write_string_with_format(2, COL_FAC, "Факультет", &header)?;
    for i in 0..teachers.len() as u16 {
        sheet.write_blank(2, COL_FIRST_TEACHER + i, &header)?;
    }

    let slots_by_day = slots_by_day(lessons);
    let index = index_lessons(lessons);

    let mut row: u32 = 3;
    for day in 1..=7i64 {
        let Some(slots) = slots_by_day.get(&day) else { continue };
        let day_start = row;
        for slot in slots {
            sheet.set_row_height(row, 18)?;
            sheet.write_string_with_format(row, COL_TIME as u16, slot, &centered)?;

            let faculties = faculties_in_slot(&index, day, slot);
            sheet.write_string_with_format(row, COL_FAC, &faculties.join(", "), &cell)?;

            for (i, teacher) in teachers.iter().enumerate() {
                let mark = if index.teacher_in_slot(day, slot, teacher.id) { "+" } else { "" };
                sheet.write_string_with_format(row, COL_FIRST_TEACHER + i as u16, mark, &centered)?;
            }
            row += 1;
        }
        let day_label = DAY_NAMES.get((day - 1) as usize).copied().unwrap_or("");
        if row - day_start > 1 {
            sheet.merge_range(day_start, COL_DAY, row - 1, COL_DAY, day_label, &centered)?;
        } else {
            sheet.write_string_with_format(day_start, COL_DAY, day_label, &centered)?;
        }
    }

    Ok(())
}

/// Индекс занятий по дню и слоту: факультеты слота и преподаватели, у которых есть пара.
struct LessonIndex {
    faculties: HashMap<(i64, String), BTreeSet<String>>,
    teachers: HashMap<(i64, String), HashSet<i64>>,
}

impl LessonIndex {
    fn teacher_in_slot(&self, day: i64, slot: &str, teacher_id: i64) -> bool {
        self.teachers
            .get(&(day, slot.to_owned()))
            .is_some_and(|set| set.contains(&teacher_id))
    }
}

fn index_lessons(lessons: &[Lesson]) -> LessonIndex {
    let mut faculties: HashMap<(i64, String), BTreeSet<String>> = HashMap::new();
    let mut teachers: HashMap<(i64, String), HashSet<i64>> = HashMap::new();
    for lesson in lessons {
        let key = (lesson.day_of_week, slot_label(&lesson.start_time, &lesson.end_time));
        faculties
            .entry(key.clone())
            .or_default()
            .extend(lesson.faculty_names.iter().filter(|n| !n.is_empty()).cloned());
        teachers.entry(key).or_default().extend(lesson.teacher_ids.iter().copied());
    }
    LessonIndex { faculties, teachers }
}

fn faculties_in_slot(index: &LessonIndex, day: i64, slot: &str) -> Vec<String> {
    index
        .faculties
        .get(&(day, slot.to_owned()))
        .map(|set| set.iter().cloned().collect())
        .unwrap_or_default()
}

/// Уникальные слоты по дням, отсортированные по времени начала.
fn slots_by_day(lessons: &[Lesson]) -> BTreeMap<i64, Vec<String>> {
    let mut map: BTreeMap<i64, BTreeSet<(u32, u32, String)>> = BTreeMap::new();
    for lesson in lessons {
        let slot = slot_label(&lesson.start_time, &lesson.end_time);
        let start = minutes(&lesson.start_time);
        let end = minutes(&lesson.end_time);
        map.entry(lesson.day_of_week).or_default().insert((start, end, slot));
    }
    map.into_iter()
        .map(|(day, set)| (day, set.into_iter().map(|(_, _, slot)| slot).collect()))
        .collect()
}

fn slot_label(start: &str, end: &str) -> String {
    format!("{}-{}", normalize_time(start), normalize_time(end))
}

fn normalize_time(value: &str) -> String {
    let mut parts = value.split(':');
    match (parts.next(), parts.next()) {
        (Some(h), Some(m)) => format!("{:0>2}:{:0>2}", h, m),
        _ => value.to_owned(),
    }
}

fn minutes(value: &str) -> u32 {
    let mut parts = value.split(':');
    let h: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let m: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    h * 60 + m
}

/// Фамилия и инициалы: «Иванов И. И.» — как в Go-версии.
fn short_name(teacher: &Teacher) -> String {
    let mut parts = Vec::new();
    let last = teacher.last_name.trim();
    if !last.is_empty() {
        parts.push(last.to_owned());
    }
    for name in [teacher.first_name.trim(), teacher.middle_name.trim()] {
        if let Some(initial) = name.chars().next() {
            parts.push(format!("{initial}."));
        }
    }
    parts.join(" ")
}

fn sanitize_file_name(value: &str) -> String {
    value
        .trim()
        .replace(' ', "_")
        .chars()
        .map(|c| if "/\\:*?\"<>|".contains(c) { '-' } else { c })
        .collect()
}
