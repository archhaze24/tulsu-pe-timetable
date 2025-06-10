use std::path::Path;
use std::fs;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn init_db(db_path: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let db_uri = format!("sqlite://{}", db_path);

    // Создаём папку и файл БД, если его нет
    let db_file = Path::new(db_path);
    if let Some(parent_dir) = db_file.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }
    if !db_file.exists() {
        fs::File::create(&db_path)?;
    }

    let pool = SqlitePoolOptions::new().connect(&db_uri).await?;
    MIGRATOR.run(&pool).await?;

    Ok(pool)
}
