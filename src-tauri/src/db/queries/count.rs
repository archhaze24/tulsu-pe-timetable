use sqlx::SqlitePool;

pub async fn increment_count(id: i64, pool: &SqlitePool) -> anyhow::Result<i64> {
    let query = sqlx::query!("SELECT count FROM test WHERE id = ?", id).fetch_one(pool).await;

    if let Err(e) = query {
        return Err(e.into());
    }

    let new_count: i64 = query?.count + 1;

    let save_result = sqlx::query!("UPDATE test SET count=? WHERE id = ?", new_count, id).execute(pool).await;
    if let Err(e) = save_result {
        return Err(e.into());
    }

    Ok(new_count)
}
