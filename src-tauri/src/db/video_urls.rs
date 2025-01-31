use sqlx::{FromRow, SqlitePool};
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(FromRow, Serialize, Deserialize)]
pub struct VideoUrl {
    source_id: i64,
    url: String,
    name: String,
    group_name: Option<String>, // group_name 是可选的
    logo: Option<String>, // logo 是可选的
}

#[tauri::command]
pub async fn get_video_urls_command(
    pool: State<'_, SqlitePool>,
    source_id: i64,
) -> Result<Vec<VideoUrl>, String> {
    let result = get_video_urls(&pool, source_id).await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_video_urls_command(
    pool: State<'_, SqlitePool>,
    source_id: i64,
    video_urls: Vec<VideoUrl>,
) -> Result<(), String> {
    add_video_urls(&pool, source_id, &video_urls)
        .await
        .map_err(|e| e.to_string())
}

pub async fn add_video_urls(
    pool: &SqlitePool,
    source_id: i64,
    video_urls: &[VideoUrl],
) -> Result<(), sqlx::Error> {
    // 开始一个事务
    let mut transaction = pool.begin().await?;

    // 先删除旧的 source_id 对应的所有记录
    sqlx::query(
        r#"
        DELETE FROM video_urls WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .execute(&mut *transaction)
    .await?;
    // 遍历 video_urls，逐个插入
    for video_url in video_urls {
        sqlx::query(
            r#"
            INSERT INTO video_urls (source_id, url, name, logo, group_name)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(video_url.source_id)
        .bind(&video_url.url)
        .bind(&video_url.name)
        .bind(&video_url.logo)
        .bind(&video_url.group_name)
        .execute(&mut *transaction)
        .await?;
    }

    // 提交事务
    transaction.commit().await?;

    Ok(())
}

pub async fn get_video_urls(
    pool: &SqlitePool,
    source_id: i64,
) -> Result<Vec<VideoUrl>, sqlx::Error> {
    let urls = sqlx::query_as::<_, VideoUrl>(
        r#"
        SELECT source_id, url, name, group_name, logo
        FROM video_urls
        WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_all(pool)
    .await?;

    Ok(urls)
}