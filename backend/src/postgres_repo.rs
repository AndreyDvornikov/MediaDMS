use sqlx::PgPool;
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct SongResponse {
    pub song_id: i32,
    pub song_name: String,
    pub author: String,
    pub album_name: String,
    pub year: i32,
    pub duration_sec: i32,
}

#[derive(Clone)]
pub struct PostgresRepository {
    pub pool: PgPool,
}

impl PostgresRepository {
    pub async fn new() -> Self {
        let url = "postgres://admin:access@localhost:5531/media_dms_db";
        let pool = PgPool::connect(url).await.expect("DB connect failed");
        Self { pool }
    }

    // ---------------- READ ----------------

    pub async fn get_songs(&self, filters: Value) -> Vec<SongResponse> {
        let mut query = String::from(r#"
            SELECT 
                s.song_id,
                s.name as song_name,
                a.name as author,
                al.name as album_name,
                al.year as year,
                s.duration as duration_sec
            FROM songs s
            JOIN albums al ON s.album_id = al.album_id
            JOIN authors a ON al.author_id = a.author_id
            WHERE 1=1
        "#);

        if let Some(author) = filters["author"].as_str() {
            query += &format!(" AND a.name ILIKE '%{}%'", author);
        }

        if let Some(song) = filters["song_name"].as_str() {
            query += &format!(" AND s.name ILIKE '%{}%'", song);
        }

        if let Some(album) = filters["album_name"].as_str() {
            query += &format!(" AND al.name ILIKE '%{}%'", album);
        }

        if let Some(range) = filters["year"].as_object() {
            if let Some(min) = range.get("min") {
                query += &format!(" AND al.year >= {}", min);
            }
            if let Some(max) = range.get("max") {
                query += &format!(" AND al.year <= {}", max);
            }
        }

        if let Some(max_dur) = filters["duration_max"].as_i64() {
            query += &format!(" AND s.duration <= {}", max_dur);
        }

        query += " ORDER BY s.song_id";

        println!("SQL: {}", query);

        sqlx::query_as::<_, SongResponse>(&query)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
    }

    // ---------------- CREATE ----------------

    pub async fn create_author(&self, name: String, bio: String) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar::<_, i32>(
            "INSERT INTO authors (name, description) VALUES ($1, $2) RETURNING author_id"
        )
        .bind(name)
        .bind(bio)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_album(&self, author_id: i32, name: String, year: i32, desc: String) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar::<_, i32>(
            "INSERT INTO albums (author_id, name, year, description) VALUES ($1, $2, $3, $4) RETURNING album_id"
        )
        .bind(author_id)
        .bind(name)
        .bind(year)
        .bind(desc)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_song(&self, album_id: i32, name: String, duration: i32, url: String) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar::<_, i32>(
            "INSERT INTO songs (album_id, name, duration, link_to_api) VALUES ($1, $2, $3, $4) RETURNING song_id"
        )
        .bind(album_id)
        .bind(name)
        .bind(duration)
        .bind(url)
        .fetch_one(&self.pool)
        .await
    }
}