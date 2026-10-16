use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// JSON-поля совпадают с Go-версией (`storage/models.go`), чтобы фронтенд не менялся.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct Direction {
    pub id: i64,
    pub name: String,
    pub is_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct Faculty {
    pub id: i64,
    pub name: String,
    pub is_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct Teacher {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: String,
    pub direction_id: i64,
    pub rate: f64,
    #[serde(default)]
    pub direction_name: String,
    pub is_archived: bool,
    pub is_guest: bool,
    #[serde(default)]
    pub is_bound: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "generated/")]
pub struct Semester {
    pub id: i64,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub is_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lesson {
    pub id: i64,
    pub semester_id: i64,
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    pub direction_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teacher_count: Option<i64>,
    #[serde(default)]
    pub faculty_ids: Vec<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub faculty_names: Vec<String>,
    #[serde(default)]
    pub teacher_ids: Vec<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub teacher_names: Vec<String>,
}

/// Входные данные занятия (внутренний формат репозитория, без JSON-контракта).
#[derive(Debug, Clone)]
pub struct LessonInput {
    pub semester_id: i64,
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    pub direction_id: i64,
    pub teacher_count: Option<i64>,
    pub faculty_ids: Vec<i64>,
    pub teacher_ids: Vec<i64>,
}
