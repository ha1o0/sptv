use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // 获取数据库路径（假设数据库文件位于应用目录下）
    let db_path = PathBuf::from("sptv.db");

    // 创建数据库连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true))
        .await?;

    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 创建视频源表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS video_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            url TEXT NOT NULL,
            is_default INTEGER DEFAULT 0,
            is_deleted INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 创建视频地址表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS video_urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL,
            video_url TEXT NOT NULL,
            name TEXT NOT NULL,
            logo TEXT,
            group_name TEXT default 'default',
            is_deleted INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (source_id) REFERENCES video_sources (id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}