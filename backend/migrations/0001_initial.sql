-- +goose Up

CREATE TABLE IF NOT EXISTS directions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    isArchived BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS faculties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    isArchived BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS teachers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    isArchived BOOLEAN NOT NULL DEFAULT FALSE,
    isGuest BOOLEAN NOT NULL DEFAULT FALSE,
    middle_name TEXT,
    direction_id INTEGER NOT NULL,
    rate REAL NOT NULL,
    FOREIGN KEY (direction_id) REFERENCES directions (id)
);

CREATE TABLE IF NOT EXISTS semesters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    isArchived BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS lessons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    semester_id INTEGER NOT NULL,
    day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 1 AND 7),
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    direction_id INTEGER NOT NULL,
    teacher_count INTEGER,
    FOREIGN KEY (semester_id) REFERENCES semesters (id),
    FOREIGN KEY (direction_id) REFERENCES directions (id)
);

CREATE TABLE IF NOT EXISTS lesson_faculties (
    lesson_id INTEGER NOT NULL,
    faculty_id INTEGER NOT NULL,
    PRIMARY KEY (lesson_id, faculty_id),
    FOREIGN KEY (lesson_id) REFERENCES lessons (id),
    FOREIGN KEY (faculty_id) REFERENCES faculties (id)
);

CREATE TABLE IF NOT EXISTS lesson_teachers (
    lesson_id INTEGER NOT NULL,
    teacher_id INTEGER NOT NULL,
    PRIMARY KEY (lesson_id, teacher_id),
    FOREIGN KEY (lesson_id) REFERENCES lessons (id),
    FOREIGN KEY (teacher_id) REFERENCES teachers (id)
);

CREATE TABLE IF NOT EXISTS semester_teachers (
    semester_id INTEGER NOT NULL,
    teacher_id INTEGER NOT NULL,
    PRIMARY KEY (semester_id, teacher_id),
    FOREIGN KEY (semester_id) REFERENCES semesters (id),
    FOREIGN KEY (teacher_id) REFERENCES teachers (id)
);


-- +goose Down

DROP TABLE IF EXISTS semester_teachers;
DROP TABLE IF EXISTS lesson_teachers;
DROP TABLE IF EXISTS lesson_faculties;
DROP TABLE IF EXISTS lessons;
DROP TABLE IF EXISTS semesters;
DROP TABLE IF EXISTS teachers;
DROP TABLE IF EXISTS faculties;
DROP TABLE IF EXISTS directions;



