use sqlx::SqlitePool;
use tauri::State;

#[tauri::command]
pub async fn add_video_url_command(pool: State<'_, SqlitePool>, source_id: i64, video_url: String) -> Result<(), String> {
    add_video_url(&pool, source_id, &video_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_video_urls_command(pool: State<'_, SqlitePool>, source_id: i64) -> Result<Vec<String>, String> {
    get_video_urls(&pool, source_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn add_video_url(pool: &SqlitePool, source_id: i64, video_url: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO video_urls (source_id, video_url) VALUES (?, ?)
        "#,
    )
    .bind(source_id)
    .bind(video_url)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_video_urls(pool: &SqlitePool, source_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let urls = sqlx::query_scalar::<_, String>(
        r#"
        SELECT video_url FROM video_urls WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_all(pool)
    .await?;

    Ok(urls)
}