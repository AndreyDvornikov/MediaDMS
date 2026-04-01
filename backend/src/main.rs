use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;

mod algorithms;
mod postgres_repo;

use postgres_repo::PostgresRepository;

#[derive(Clone)]
struct AppState {
    repo: Arc<PostgresRepository>,
}

#[derive(Deserialize)]
struct QueryRequest {
    method: String,
    entity: String,
    #[serde(default)]
    filters: Value,
    #[serde(default)]
    sort: Value,
    #[serde(default)]
    data: Value,
}

#[tokio::main]
async fn main() {
    let repo = Arc::new(PostgresRepository::new().await);
    let state = AppState { repo };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/query", post(query))
        .with_state(state);

    println!("Server started at http://127.0.0.1:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn query(
    State(state): State<AppState>,
    Json(body): Json<QueryRequest>,
) -> Json<Value> {
    match (body.method.as_str(), body.entity.as_str()) {
        ("read", "song") => read_songs(&state.repo, &body).await,
        ("read", "album") => read_albums(&state.repo, &body).await,
        ("read", "author") => read_authors(&state.repo, &body).await,
        ("write", "author") => create_author(&state.repo, &body.data).await,
        ("write", "album") => create_album(&state.repo, &body.data).await,
        ("write", "song") => create_song(&state.repo, &body.data).await,
        _ => error_response(1, "Unknown request"),
    }
}

async fn read_songs(repo: &PostgresRepository, body: &QueryRequest) -> Json<Value> {
    match repo
        .get_songs(
            &body.filters,
            body.sort.get("field").and_then(Value::as_str),
            body.sort.get("order").and_then(Value::as_str),
        )
        .await
    {
        Ok(items) => Json(json!({ "error": 0, "data": { "items": items } })),
        Err(err) => error_response(3, &err.to_string()),
    }
}

async fn read_albums(repo: &PostgresRepository, body: &QueryRequest) -> Json<Value> {
    match repo
        .get_albums(
            &body.filters,
            body.sort.get("field").and_then(Value::as_str),
            body.sort.get("order").and_then(Value::as_str),
        )
        .await
    {
        Ok(items) => Json(json!({ "error": 0, "data": { "items": items } })),
        Err(err) => error_response(3, &err.to_string()),
    }
}

async fn read_authors(repo: &PostgresRepository, body: &QueryRequest) -> Json<Value> {
    match repo.get_authors(&body.filters).await {
        Ok(items) => Json(json!({ "error": 0, "data": { "items": items } })),
        Err(err) => error_response(3, &err.to_string()),
    }
}

async fn create_author(repo: &PostgresRepository, data: &Value) -> Json<Value> {
    let Some(name) = data.get("author_name").and_then(Value::as_str) else {
        return error_response(4, "author_name is required");
    };
    let bio = data.get("bio").and_then(Value::as_str).unwrap_or("");

    let Some(raw_images) = data.get("images_binaries").and_then(Value::as_array) else {
        return error_response(4, "images_binaries must be an array");
    };

    if raw_images.len() > 6 {
        return error_response(4, "images_binaries length must be <= 6");
    }

    let mut images = Vec::with_capacity(raw_images.len());
    for value in raw_images {
        let Some(encoded) = value.as_str() else {
            return error_response(4, "images_binaries items must be strings");
        };
        match decode_base64(encoded) {
            Ok(bytes) => images.push(bytes),
            Err(err) => return error_response(4, &err),
        }
    }

    match repo.create_author(name, bio, &images).await {
        Ok(created_id) => success_created(created_id),
        Err(err) => error_response(3, &err.to_string()),
    }
}

async fn create_album(repo: &PostgresRepository, data: &Value) -> Json<Value> {
    let Some(author_id) = data.get("author_id").and_then(Value::as_i64) else {
        return error_response(4, "author_id is required");
    };
    let Some(album_name) = data.get("album_name").and_then(Value::as_str) else {
        return error_response(4, "album_name is required");
    };
    let Some(year) = data.get("year").and_then(Value::as_i64) else {
        return error_response(4, "year is required");
    };

    match repo.author_exists(author_id as i32).await {
        Ok(true) => {}
        Ok(false) => return error_response(2, "Author not found"),
        Err(err) => return error_response(3, &err.to_string()),
    }

    let description = data.get("description").and_then(Value::as_str).unwrap_or("");
    let cover_binary = match data.get("cover_binary").and_then(Value::as_str) {
        Some(value) if !value.is_empty() => match decode_base64(value) {
            Ok(bytes) => Some(bytes),
            Err(err) => return error_response(4, &err),
        },
        _ => None,
    };

    match repo
        .create_album(author_id as i32, album_name, year as i32, description, cover_binary)
        .await
    {
        Ok(created_id) => success_created(created_id),
        Err(err) => error_response(3, &err.to_string()),
    }
}

async fn create_song(repo: &PostgresRepository, data: &Value) -> Json<Value> {
    let Some(album_id) = data.get("album_id").and_then(Value::as_i64) else {
        return error_response(4, "album_id is required");
    };
    let Some(song_name) = data.get("song_name").and_then(Value::as_str) else {
        return error_response(4, "song_name is required");
    };
    let Some(duration_sec) = data.get("duration_sec").and_then(Value::as_i64) else {
        return error_response(4, "duration_sec is required");
    };
    let audio_url = data.get("audio_url").and_then(Value::as_str).unwrap_or("");

    match repo.album_year(album_id as i32).await {
        Ok(Some(_)) => {}
        Ok(None) => return error_response(2, "Album not found"),
        Err(err) => return error_response(3, &err.to_string()),
    }

    match repo
        .create_song(album_id as i32, song_name, duration_sec as i32, audio_url)
        .await
    {
        Ok(created_id) => success_created(created_id),
        Err(err) => error_response(3, &err.to_string()),
    }
}

fn success_created(created_id: i32) -> Json<Value> {
    Json(json!({
        "error": 0,
        "status": "success",
        "created_id": created_id
    }))
}

fn error_response(code: u8, message: &str) -> Json<Value> {
    Json(json!({
        "error": code,
        "error_message": message
    }))
}

fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    STANDARD
        .decode(value)
        .map_err(|_| "Invalid base64 payload".to_string())
}
