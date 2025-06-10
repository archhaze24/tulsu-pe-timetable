use sqlx::SqlitePool;

mod init;
pub mod queries;

pub struct Db {
    pub pool: SqlitePool
}

impl Db {
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let pool = init::init_db(path).await?;

        Ok(Self { pool })
    }
}