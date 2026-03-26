use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use sqlx::{PgPool, Row};

use crate::error::RepoError;
use crate::models::{AlbumRecord, SongRecord};

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn all_songs(&self) -> Result<Vec<SongRecord>, RepoError>;
    async fn all_albums(&self) -> Result<Vec<AlbumRecord>, RepoError>;
    async fn author_images(&self, author: &str) -> Result<Vec<String>, RepoError>;
}

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MediaRepository for PgRepository {
    async fn all_songs(&self) -> Result<Vec<SongRecord>, RepoError> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.song_id,
                a.name AS author,
                al.name AS album_name,
                s.name AS song_name,
                al.year AS year,
                s.duration AS duration_sec
            FROM songs s
            JOIN albums al ON al.album_id = s.album_id
            JOIN authors a ON a.author_id = al.author_id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::Read(format!("failed to read songs: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(SongRecord {
                song_id: row
                    .try_get::<i32, _>("song_id")
                    .map_err(|e| RepoError::Read(format!("song_id mapping error: {e}")))?
                    as u32,
                author: row
                    .try_get::<String, _>("author")
                    .map_err(|e| RepoError::Read(format!("author mapping error: {e}")))?,
                album_name: row
                    .try_get::<String, _>("album_name")
                    .map_err(|e| RepoError::Read(format!("album_name mapping error: {e}")))?,
                song_name: row
                    .try_get::<String, _>("song_name")
                    .map_err(|e| RepoError::Read(format!("song_name mapping error: {e}")))?,
                year: row
                    .try_get::<i32, _>("year")
                    .map_err(|e| RepoError::Read(format!("year mapping error: {e}")))?
                    as u32,
                duration_sec: row
                    .try_get::<i32, _>("duration_sec")
                    .map_err(|e| RepoError::Read(format!("duration_sec mapping error: {e}")))?
                    as u32,
            });
        }

        Ok(out)
    }

    async fn all_albums(&self) -> Result<Vec<AlbumRecord>, RepoError> {
        let rows = sqlx::query(
            r#"
            SELECT
                al.album_id,
                a.name AS author,
                al.name AS album_name,
                COALESCE(al.description, '') AS description,
                al.cover_data,
                al.year
            FROM albums al
            JOIN authors a ON a.author_id = al.author_id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::Read(format!("failed to read albums: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let cover_bytes = row
                .try_get::<Option<Vec<u8>>, _>("cover_data")
                .map_err(|e| RepoError::Read(format!("cover_data mapping error: {e}")))?;

            let cover_url = match cover_bytes {
                Some(bytes) if !bytes.is_empty() => {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                    format!("data:image/jpeg;base64,{b64}")
                }
                _ => String::new(),
            };

            out.push(AlbumRecord {
                album_id: row
                    .try_get::<i32, _>("album_id")
                    .map_err(|e| RepoError::Read(format!("album_id mapping error: {e}")))?
                    as u32,
                author: row
                    .try_get::<String, _>("author")
                    .map_err(|e| RepoError::Read(format!("author mapping error: {e}")))?,
                album_name: row
                    .try_get::<String, _>("album_name")
                    .map_err(|e| RepoError::Read(format!("album_name mapping error: {e}")))?,
                description: row
                    .try_get::<String, _>("description")
                    .map_err(|e| RepoError::Read(format!("description mapping error: {e}")))?,
                cover_url,
                year: row
                    .try_get::<i32, _>("year")
                    .map_err(|e| RepoError::Read(format!("year mapping error: {e}")))?
                    as u32,
            });
        }

        Ok(out)
    }

    async fn author_images(&self, author: &str) -> Result<Vec<String>, RepoError> {
        let rows = sqlx::query(
            r#"
            SELECT ai.image_data
            FROM author_images ai
            JOIN authors a ON a.author_id = ai.author_id
            WHERE LOWER(a.name) = LOWER($1)
            ORDER BY ai.image_id
            "#,
        )
        .bind(author)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::Read(format!("failed to read author images: {e}")))?;

        let mut images = Vec::with_capacity(rows.len());
        for row in rows {
            let bytes = row
                .try_get::<Vec<u8>, _>("image_data")
                .map_err(|e| RepoError::Read(format!("image_data mapping error: {e}")))?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
            images.push(format!("data:image/jpeg;base64,{b64}"));
        }

        Ok(images)
    }
}

#[derive(Clone)]
pub struct InMemoryRepository {
    songs: Arc<Vec<SongRecord>>,
    albums: Arc<Vec<AlbumRecord>>,
}

impl InMemoryRepository {
    pub fn new_seeded() -> Self {
        let albums = vec![
            AlbumRecord {
                album_id: 11,
                author: "Linkin Park".to_string(),
                album_name: "Meteora".to_string(),
                description: "Studio album by Linkin Park".to_string(),
                cover_url: "https://img.example/meteora.jpg".to_string(),
                year: 2003,
            },
            AlbumRecord {
                album_id: 12,
                author: "Linkin Park".to_string(),
                album_name: "Minutes to Midnight".to_string(),
                description: "Third studio album".to_string(),
                cover_url: "https://img.example/minutes.jpg".to_string(),
                year: 2007,
            },
            AlbumRecord {
                album_id: 31,
                author: "Pelmen".to_string(),
                album_name: "Cold Dumpling".to_string(),
                description: "Demo album".to_string(),
                cover_url: "https://img.example/pelmen-1.jpg".to_string(),
                year: 2019,
            },
            AlbumRecord {
                album_id: 32,
                author: "Pelmen".to_string(),
                album_name: "Warm Dumpling".to_string(),
                description: "Second album".to_string(),
                cover_url: "https://img.example/pelmen-2.jpg".to_string(),
                year: 2021,
            },
            AlbumRecord {
                album_id: 71,
                author: "Popugay".to_string(),
                album_name: "Jungle".to_string(),
                description: "Tropical beats collection".to_string(),
                cover_url: "https://img.example/jungle.jpg".to_string(),
                year: 2020,
            },
        ];

        let songs = vec![
            SongRecord {
                song_id: 10,
                author: "Linkin Park".to_string(),
                album_name: "Meteora".to_string(),
                song_name: "Numb".to_string(),
                year: 2003,
                duration_sec: 185,
            },
            SongRecord {
                song_id: 25,
                author: "Linkin Park".to_string(),
                album_name: "Minutes to Midnight".to_string(),
                song_name: "Given Up".to_string(),
                year: 2007,
                duration_sec: 189,
            },
            SongRecord {
                song_id: 73,
                author: "Linkin Park".to_string(),
                album_name: "One More Light".to_string(),
                song_name: "Invisible".to_string(),
                year: 2017,
                duration_sec: 214,
            },
            SongRecord {
                song_id: 44,
                author: "Popugay".to_string(),
                album_name: "Jungle".to_string(),
                song_name: "Banana Echo".to_string(),
                year: 2020,
                duration_sec: 204,
            },
            SongRecord {
                song_id: 45,
                author: "Popugay".to_string(),
                album_name: "Jungle".to_string(),
                song_name: "Canopy Rush".to_string(),
                year: 2020,
                duration_sec: 240,
            },
        ];

        Self {
            songs: Arc::new(songs),
            albums: Arc::new(albums),
        }
    }
}

#[async_trait]
impl MediaRepository for InMemoryRepository {
    async fn all_songs(&self) -> Result<Vec<SongRecord>, RepoError> {
        Ok((*self.songs).clone())
    }

    async fn all_albums(&self) -> Result<Vec<AlbumRecord>, RepoError> {
        Ok((*self.albums).clone())
    }

    async fn author_images(&self, author: &str) -> Result<Vec<String>, RepoError> {
        let known = vec![
            "https://img.example/author-1.jpg",
            "https://img.example/author-2.jpg",
            "https://img.example/author-3.jpg",
            "https://img.example/author-4.jpg",
            "https://img.example/author-5.jpg",
            "https://img.example/author-6.jpg",
            "https://img.example/author-7.jpg",
        ];

        let has_author = self
            .albums
            .iter()
            .any(|album| album.author.eq_ignore_ascii_case(author))
            || self
                .songs
                .iter()
                .any(|song| song.author.eq_ignore_ascii_case(author));

        if !has_author {
            return Ok(Vec::new());
        }

        Ok(known
            .into_iter()
            .map(|url| format!("{url}?a={author}"))
            .collect())
    }
}
