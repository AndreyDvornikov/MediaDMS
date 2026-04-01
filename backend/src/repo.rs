use std::sync::Arc;

use crate::error::RepoError;
use crate::models::{AlbumRecord, SongRecord};

pub trait MediaRepository: Send + Sync {
    fn all_songs(&self) -> Result<Vec<SongRecord>, RepoError>;
    fn all_albums(&self) -> Result<Vec<AlbumRecord>, RepoError>;
    fn author_images(&self, author: &str) -> Result<Vec<String>, RepoError>;
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

impl MediaRepository for InMemoryRepository {
    fn all_songs(&self) -> Result<Vec<SongRecord>, RepoError> {
        Ok((*self.songs).clone())
    }

    fn all_albums(&self) -> Result<Vec<AlbumRecord>, RepoError> {
        Ok((*self.albums).clone())
    }

    fn author_images(&self, author: &str) -> Result<Vec<String>, RepoError> {
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
