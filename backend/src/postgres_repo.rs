use std::env;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Serialize;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

#[derive(sqlx::FromRow)]
struct SongRow {
    song_id: i32,
    song_name: String,
    author: String,
    album_name: String,
    year: i32,
    duration_sec: i32,
    audio_url: Option<String>,
    cover_data: Option<Vec<u8>>,
}

#[derive(sqlx::FromRow)]
struct AlbumRow {
    album_id: i32,
    author_id: i32,
    author: String,
    album_name: String,
    description: Option<String>,
    year: i32,
    cover_data: Option<Vec<u8>>,
}

#[derive(sqlx::FromRow)]
struct AuthorRow {
    author_id: i32,
    author: String,
    description: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct SongResponse {
    pub song_id: i32,
    pub song_name: String,
    pub author: String,
    pub album_name: String,
    pub year: i32,
    pub duration_sec: i32,
    pub audio_url: Option<String>,
    pub cover_binary: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct AlbumResponse {
    pub album_id: i32,
    pub author_id: i32,
    pub author: String,
    pub album_name: String,
    pub description: String,
    pub year: i32,
    pub cover_binary: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct AuthorResponse {
    pub author_id: i32,
    pub author: String,
    pub description: String,
    pub images_binaries: Vec<String>,
}

#[derive(Clone)]
pub struct PostgresRepository {
    pub pool: PgPool,
}

impl PostgresRepository {
    pub async fn new() -> Self {
        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://admin:access@localhost:5531/media_dms_db".to_string());
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .expect("DB connect failed");
        Self { pool }
    }

    pub async fn get_songs(
        &self,
        filters: &Value,
        sort_field: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<Vec<SongResponse>, sqlx::Error> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT s.song_id, s.name AS song_name, a.name AS author, al.name AS album_name, al.year AS year, s.duration AS duration_sec, s.link_to_api AS audio_url, al.cover_data FROM songs s JOIN albums al ON s.album_id = al.album_id JOIN authors a ON al.author_id = a.author_id WHERE 1=1",
        );

        if let Some(range) = filters.get("song_id").and_then(Value::as_object) {
            if let Some(min) = range.get("min").and_then(Value::as_i64) {
                builder.push(" AND s.song_id >= ");
                builder.push_bind(min as i32);
            }
            if let Some(max) = range.get("max").and_then(Value::as_i64) {
                builder.push(" AND s.song_id <= ");
                builder.push_bind(max as i32);
            }
        }

        if let Some(author) = filters.get("author").and_then(Value::as_str) {
            builder.push(" AND a.name ILIKE ");
            builder.push_bind(like(author));
        }

        if let Some(song) = filters.get("song_name").and_then(Value::as_str) {
            builder.push(" AND s.name ILIKE ");
            builder.push_bind(like(song));
        }

        if let Some(album) = filters.get("album_name").and_then(Value::as_str) {
            builder.push(" AND al.name ILIKE ");
            builder.push_bind(like(album));
        }

        if let Some(range) = filters.get("year").and_then(Value::as_object) {
            if let Some(min) = range.get("min").and_then(Value::as_i64) {
                builder.push(" AND al.year >= ");
                builder.push_bind(min as i32);
            }
            if let Some(max) = range.get("max").and_then(Value::as_i64) {
                builder.push(" AND al.year <= ");
                builder.push_bind(max as i32);
            }
        }

        if let Some(max_dur) = filters.get("duration_max").and_then(Value::as_i64) {
            builder.push(" AND s.duration <= ");
            builder.push_bind(max_dur as i32);
        }

        builder.push(" ORDER BY ");
        builder.push(song_sort_column(sort_field));
        builder.push(" ");
        builder.push(sort_direction(sort_order));

        let rows = builder
            .build_query_as::<SongRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(map_song).collect())
    }

    pub async fn get_albums(
        &self,
        filters: &Value,
        sort_field: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<Vec<AlbumResponse>, sqlx::Error> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT al.album_id, al.author_id, a.name AS author, al.name AS album_name, al.description, al.year, al.cover_data FROM albums al JOIN authors a ON al.author_id = a.author_id WHERE 1=1",
        );

        if let Some(range) = filters.get("album_id").and_then(Value::as_object) {
            if let Some(min) = range.get("min").and_then(Value::as_i64) {
                builder.push(" AND al.album_id >= ");
                builder.push_bind(min as i32);
            }
            if let Some(max) = range.get("max").and_then(Value::as_i64) {
                builder.push(" AND al.album_id <= ");
                builder.push_bind(max as i32);
            }
        }

        if let Some(author) = filters.get("author").and_then(Value::as_str) {
            builder.push(" AND a.name ILIKE ");
            builder.push_bind(like(author));
        }

        if let Some(album) = filters.get("album_name").and_then(Value::as_str) {
            builder.push(" AND al.name ILIKE ");
            builder.push_bind(like(album));
        }

        if let Some(range) = filters.get("year").and_then(Value::as_object) {
            if let Some(min) = range.get("min").and_then(Value::as_i64) {
                builder.push(" AND al.year >= ");
                builder.push_bind(min as i32);
            }
            if let Some(max) = range.get("max").and_then(Value::as_i64) {
                builder.push(" AND al.year <= ");
                builder.push_bind(max as i32);
            }
        }

        builder.push(" ORDER BY ");
        builder.push(album_sort_column(sort_field));
        builder.push(" ");
        builder.push(sort_direction(sort_order));

        let rows = builder
            .build_query_as::<AlbumRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(map_album).collect())
    }

    pub async fn get_authors(&self, filters: &Value) -> Result<Vec<AuthorResponse>, sqlx::Error> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT author_id, name AS author, description FROM authors WHERE 1=1",
        );

        if let Some(author) = filters.get("author").and_then(Value::as_str) {
            builder.push(" AND name ILIKE ");
            builder.push_bind(like(author));
        }

        builder.push(" ORDER BY author_id ASC");

        let rows = builder
            .build_query_as::<AuthorRow>()
            .fetch_all(&self.pool)
            .await?;

        let mut authors = Vec::with_capacity(rows.len());
        for row in rows {
            let image_rows = sqlx::query(
                "SELECT image_data FROM author_images WHERE author_id = $1 ORDER BY image_id ASC LIMIT 6",
            )
            .bind(row.author_id)
            .fetch_all(&self.pool)
            .await?;

            let images_binaries = image_rows
                .into_iter()
                .filter_map(|image| image.try_get::<Vec<u8>, _>("image_data").ok())
                .map(|bytes| STANDARD.encode(bytes))
                .collect();

            authors.push(AuthorResponse {
                author_id: row.author_id,
                author: row.author,
                description: row.description.unwrap_or_default(),
                images_binaries,
            });
        }

        Ok(authors)
    }

    pub async fn create_author(
        &self,
        name: &str,
        bio: &str,
        images: &[Vec<u8>],
    ) -> Result<i32, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let author_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO authors (name, description) VALUES ($1, $2) RETURNING author_id",
        )
        .bind(name)
        .bind(bio)
        .fetch_one(&mut *tx)
        .await?;

        for image in images {
            sqlx::query("INSERT INTO author_images (author_id, image_data) VALUES ($1, $2)")
                .bind(author_id)
                .bind(image)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(author_id)
    }

    pub async fn author_exists(&self, author_id: i32) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM authors WHERE author_id = $1)")
            .bind(author_id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn create_album(
        &self,
        author_id: i32,
        name: &str,
        year: i32,
        desc: &str,
        cover: Option<Vec<u8>>,
    ) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar::<_, i32>(
            "INSERT INTO albums (author_id, name, year, description, cover_data) VALUES ($1, $2, $3, $4, $5) RETURNING album_id",
        )
        .bind(author_id)
        .bind(name)
        .bind(year)
        .bind(desc)
        .bind(cover)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn album_year(&self, album_id: i32) -> Result<Option<i32>, sqlx::Error> {
        sqlx::query_scalar::<_, i32>("SELECT year FROM albums WHERE album_id = $1")
            .bind(album_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create_song(
        &self,
        album_id: i32,
        name: &str,
        duration: i32,
        url: &str,
    ) -> Result<i32, sqlx::Error> {
        sqlx::query_scalar::<_, i32>(
            "INSERT INTO songs (album_id, name, duration, link_to_api) VALUES ($1, $2, $3, $4) RETURNING song_id",
        )
        .bind(album_id)
        .bind(name)
        .bind(duration)
        .bind(url)
        .fetch_one(&self.pool)
        .await
    }
}

fn map_song(row: SongRow) -> SongResponse {
    SongResponse {
        song_id: row.song_id,
        song_name: row.song_name,
        author: row.author,
        album_name: row.album_name,
        year: row.year,
        duration_sec: row.duration_sec,
        audio_url: row.audio_url,
        cover_binary: row.cover_data.map(|bytes| STANDARD.encode(bytes)),
    }
}

fn map_album(row: AlbumRow) -> AlbumResponse {
    AlbumResponse {
        album_id: row.album_id,
        author_id: row.author_id,
        author: row.author,
        album_name: row.album_name,
        description: row.description.unwrap_or_default(),
        year: row.year,
        cover_binary: row.cover_data.map(|bytes| STANDARD.encode(bytes)),
    }
}

fn like(value: &str) -> String {
    format!("%{}%", value)
}

fn song_sort_column(sort_field: Option<&str>) -> &'static str {
    match sort_field.unwrap_or("") {
        "year" => "al.year",
        "song_name" => "LOWER(s.name)",
        "album_name" => "LOWER(al.name)",
        "author" => "LOWER(a.name)",
        "duration_sec" => "s.duration",
        "song_id" => "s.song_id",
        _ => "s.song_id",
    }
}

fn album_sort_column(sort_field: Option<&str>) -> &'static str {
    match sort_field.unwrap_or("") {
        "year" => "al.year",
        "album_name" => "LOWER(al.name)",
        "author" => "LOWER(a.name)",
        "album_id" => "al.album_id",
        _ => "al.album_id",
    }
}

fn sort_direction(sort_order: Option<&str>) -> &'static str {
    match sort_order.unwrap_or("asc").to_ascii_lowercase().as_str() {
        "desc" => "DESC",
        _ => "ASC",
    }
}
