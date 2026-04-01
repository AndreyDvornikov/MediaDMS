use axum::{routing::post, Router, Json};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use axum::extract::State;

mod postgres_repo;
mod algorithms;

use postgres_repo::{PostgresRepository, SongResponse};
use algorithms::{binary_search, radix_sort, A2Tree};

#[derive(Clone)]
struct AppState {
    repo: Arc<PostgresRepository>,
}

#[tokio::main]
async fn main() {
    let repo = Arc::new(PostgresRepository::new().await);
    let state = AppState { repo };

    let app = Router::new()
        .route("/api/v1/query", post(query))
        .with_state(state);

    println!("Server started at http://127.0.0.1:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn query(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<Value> {

    let method = body["method"].as_str().unwrap_or("");
    let entity = body["entity"].as_str().unwrap_or("");

    if method == "read" && entity == "song" {

        let mut songs = state.repo.get_songs(body["filters"].clone()).await;

        // =========================
        // 🔥 RADIX SORT (оставили как было)
        // =========================
        radix_sort(&mut songs);

        // =========================
        // 🔥 BINARY SEARCH (ID)
        // =========================
        if let Some(min) = body["filters"]["song_id"]["min"].as_i64() {
            if let Some(max) = body["filters"]["song_id"]["max"].as_i64() {

                songs.sort_by_key(|s| s.song_id);

                let left = binary_search(&songs, min as i32);
                let right = binary_search(&songs, max as i32);

                if let (Some(l), Some(r)) = (left, right) {
                    songs = songs[l..=r].to_vec();
                } else {
                    songs.clear();
                }
            }
        }

        // =========================
        // 🔥 A2 TREE
        // =========================
        if let Some(name) = body["filters"]["song_name"].as_str() {
            let mut tree = A2Tree::new();
            for s in songs.clone() {
                tree.insert(s.song_name.clone(), s);
            }
            songs = tree.search(name.to_string());
        }

        if let Some(album) = body["filters"]["album_name"].as_str() {
            let mut tree = A2Tree::new();
            for s in songs.clone() {
                tree.insert(s.album_name.clone(), s);
            }
            songs = tree.search(album.to_string());
        }

        if let Some(author) = body["filters"]["author"].as_str() {
            let mut tree = A2Tree::new();
            for s in songs.clone() {
                tree.insert(s.author.clone(), s);
            }
            songs = tree.search(author.to_string());
        }

        // =========================
        // 🔥 duration (точное совпадение)
        // =========================
        if let Some(dur) = body["filters"]["duration_max"].as_i64() {
            songs = songs.into_iter()
                .filter(|s| s.duration_sec == dur as i32)
                .collect();
        }

        // =========================
        // 🔥 ДОБАВЛЕННАЯ СОРТИРОВКА (UI)
        // =========================
        let sort_field = body["sort"]["field"].as_str().unwrap_or("");
        let sort_order = body["sort"]["order"].as_str().unwrap_or("asc");

        match sort_field {

            "song_id" => {
                radix_sort(&mut songs);
            }

            "year" => {
                songs.sort_by_key(|s| s.year);
            }

            "duration_sec" => {
                songs.sort_by_key(|s| s.duration_sec);
            }

            "song_name" => {
                songs.sort_by(|a, b| a.song_name.to_lowercase().cmp(&b.song_name.to_lowercase()));
            }

            "author" => {
                songs.sort_by(|a, b| a.author.to_lowercase().cmp(&b.author.to_lowercase()));
            }

            _ => {}
        }

        if sort_order == "desc" {
            songs.reverse();
        }

        return Json(serde_json::json!({
            "error": 0,
            "data": { "items": songs }
        }));
    }

    Json(serde_json::json!({
        "error": 1,
        "error_message": "Unknown request"
    }))
}