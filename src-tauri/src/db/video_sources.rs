use sqlx::SqlitePool;
use tauri::State;
use sqlx::FromRow;
use serde::Serialize;

#[derive(FromRow, Serialize)]
pub struct VideoSource {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub is_default: i64,
    pub is_deleted: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn add_video_source_command(pool: State<'_, SqlitePool>, name: String, url: String) -> Result<i64, String> {
    add_video_source(&pool, &name, &url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_video_sources_command(pool: State<'_, SqlitePool>) -> Result<Vec<VideoSource>, String> {
    get_video_sources(&pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_video_source_command(
    pool: State<'_, SqlitePool>,
    id: i64,
    name: String,
    url: String,
) -> Result<(), String> {
    update_video_source(&pool, id, &name, &url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_video_source_command(pool: State<'_, SqlitePool>, id: i64) -> Result<(), String> {
    delete_video_source(&pool, id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn update_video_source(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    url: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE video_sources
        SET name = ?, url = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(url)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_video_source(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE video_sources
        SET is_deleted = 1, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_video_sources(pool: &SqlitePool) -> Result<Vec<VideoSource>, sqlx::Error> {
    let sources = sqlx::query_as::<_, VideoSource>(
        r#"
        SELECT * FROM video_sources WHERE is_deleted = 0
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(sources)
}

pub async fn add_video_source(pool: &SqlitePool, name: &str, url: &str) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO video_sources (name, url) VALUES (?, ?)
        "#,
    )
    .bind(name)
    .bind(url)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}