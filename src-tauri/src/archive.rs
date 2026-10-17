//! Упаковка файлов журнала и резервных копий в zip.
//!
//! Исходный файл удаляется только после полной записи архива. Если архив уже есть,
//! файл добавляется в него отдельной записью, поэтому повторная упаковка не теряет данные.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use chrono::Local;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Упаковывает `source` в `<имя>.zip` рядом с ним и удаляет исходный файл.
pub fn zip_file(source: &Path) -> io::Result<PathBuf> {
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "некорректное имя файла"))?;
    let target = source.with_file_name(format!("{file_name}.zip"));
    let existed = target.exists();

    let result = write_entry(source, &target, file_name);
    match result {
        Ok(()) => {
            fs::remove_file(source)?;
            Ok(target)
        }
        Err(e) => {
            // Частично записанный новый архив не оставляем: исходник на месте, данные целы.
            if !existed {
                let _ = fs::remove_file(&target);
            }
            Err(e)
        }
    }
}

fn write_entry(source: &Path, target: &Path, file_name: &str) -> io::Result<()> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    // Имя записи в архиве должно быть уникальным: при добавлении в существующий архив ставим метку времени.
    let entry_name = if target.exists() && has_entry_clash(target, file_name) {
        let stamp = Local::now().format("%Y%m%d-%H%M%S%3f");
        let (stem, ext) = file_name.rsplit_once('.').unwrap_or((file_name, ""));
        if ext.is_empty() { format!("{stem}-{stamp}") } else { format!("{stem}-{stamp}.{ext}") }
    } else {
        file_name.to_owned()
    };

    let mut writer = if target.exists() {
        let file = OpenOptions::new().read(true).write(true).open(target)?;
        ZipWriter::new_append(file)?
    } else {
        ZipWriter::new(File::create(target)?)
    };

    writer.start_file(entry_name, options)?;
    io::copy(&mut File::open(source)?, &mut writer)?;
    writer.finish()?;
    Ok(())
}

fn has_entry_clash(target: &Path, entry: &str) -> bool {
    let Ok(file) = File::open(target) else { return false };
    let Ok(archive) = zip::ZipArchive::new(file) else { return false };
    archive.file_names().any(|n| n == entry)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tulsu-archive-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn read_entry(archive: &Path, entry: &str) -> String {
        let mut zip = zip::ZipArchive::new(File::open(archive).unwrap()).unwrap();
        let mut text = String::new();
        zip.by_name(entry).unwrap().read_to_string(&mut text).unwrap();
        text
    }

    #[test]
    fn zip_keeps_content_and_removes_source() {
        let dir = scratch_dir("single");
        let source = dir.join("app-2026-10-16.log");
        fs::write(&source, "строка журнала\n").unwrap();

        let target = zip_file(&source).unwrap();

        assert_eq!(target, dir.join("app-2026-10-16.log.zip"));
        assert!(!source.exists());
        assert_eq!(read_entry(&target, "app-2026-10-16.log"), "строка журнала\n");
    }

    #[test]
    fn second_file_is_appended_under_unique_name() {
        let dir = scratch_dir("append");
        let target = dir.join("app-2026-10-16.log.zip");
        fs::write(dir.join("app-2026-10-16.log"), "первый\n").unwrap();
        zip_file(&dir.join("app-2026-10-16.log")).unwrap();

        fs::write(dir.join("app-2026-10-16.log"), "второй\n").unwrap();
        zip_file(&dir.join("app-2026-10-16.log")).unwrap();

        let names: Vec<String> = zip::ZipArchive::new(File::open(&target).unwrap())
            .unwrap()
            .file_names()
            .map(str::to_owned)
            .collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"app-2026-10-16.log".to_owned()));
        assert!(names.iter().any(|n| n != "app-2026-10-16.log" && n.starts_with("app-2026-10-16-")));
    }
}
