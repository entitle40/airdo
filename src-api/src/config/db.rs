use anyhow::Ok;
use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous}, Pool, Sqlite};

use crate::{mapper::health_check_mapper, util::file_util};

use super::app_config::Config;

pub async fn init() -> anyhow::Result<Pool<Sqlite>> {
    let db_path = "data/airdo.db";
    file_util::create_file_if_not_exist(db_path)?;

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Off)
        .create_if_missing(true)
        .foreign_keys(false);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await.unwrap_or_else(|e| {
                       tracing::error!("migrations db error: {:?}", e);
                   });

    Ok(pool)
}

pub async fn clear(_config: &Config, pool: &Pool<Sqlite>) {
    let _ = health_check_mapper::delete_before_time(15, pool).await;
}