mod error;
mod logging;
mod models;
mod repo;
mod service;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

use crate::error::{ApiError, ErrorCode};
use crate::logging::RequestLogger;
use crate::models::{ApiRequest, ApiResponse, Entity, HealthResponse};
use crate::repo::PgRepository;
use crate::service::{error_response, SearchService};

#[derive(Clone)]
struct AppState {
    service: SearchService,
    logger: Arc<RequestLogger>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = build_database_url();

    let pool = connect_postgres_with_retry(&database_url)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "connect to postgres failed after retries: {err}. \
Ensure DB is running: docker-compose -f database/docker-compose.yml up -d"
            )
        });

    let repo = Arc::new(PgRepository::new(pool));
    let service = SearchService::new(repo);
    let logger = Arc::new(RequestLogger::new(5000));

    let state = AppState { service, logger };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/query", post(query))
        .route("/api/v1/logs", get(logs))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:8080".parse().expect("valid listen address");

    info!("Starting media_dms_api at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind tcp listener");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("run axum server");
}

fn build_database_url() -> String {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return with_sslmode_disable_if_missing(url);
        }
    }

    let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "admin".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "access".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "media_dms_db".to_string());
    let port = std::env::var("DB_PORT").unwrap_or_else(|_| "5531".to_string());

    with_sslmode_disable_if_missing(format!("postgres://{user}:{password}@localhost:{port}/{db}"))
}

fn with_sslmode_disable_if_missing(url: String) -> String {
    if url.contains("sslmode=") {
        return url;
    }

    if url.contains('?') {
        format!("{url}&sslmode=disable")
    } else {
        format!("{url}?sslmode=disable")
    }
}

async fn connect_postgres_with_retry(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    let attempts = 15_u32;
    let delay = Duration::from_secs(2);
    let mut last_err: Option<sqlx::Error> = None;

    for attempt in 1..=attempts {
        match PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await
        {
            Ok(pool) => {
                info!("Connected to PostgreSQL on attempt {}", attempt);
                return Ok(pool);
            }
            Err(err) => {
                error!(
                    "PostgreSQL connection attempt {}/{} failed: {}",
                    attempt, attempts, err
                );
                last_err = Some(err);
                tokio::time::sleep(delay).await;
            }
        }
    }

    Err(last_err.unwrap_or(sqlx::Error::PoolTimedOut))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn logs(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.logger.list()))
}

async fn query(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    body: String,
) -> impl IntoResponse {
    let parsed: Result<ApiRequest, _> = serde_json::from_str(&body);

    let request = match parsed {
        Ok(req) => req,
        Err(err) => {
            let msg = format!("Invalid JSON payload: {err}");
            let response = ApiResponse {
                entity: None,
                filters: Default::default(),
                sort: Default::default(),
                error: ErrorCode::InvalidClientRequest.as_u8(),
                error_message: Some(msg.clone()),
                data: None,
            };

            state.logger.push(
                addr.ip().to_string(),
                Entity::Song,
                ErrorCode::InvalidClientRequest.as_u8(),
                Some(msg),
            );

            return (StatusCode::BAD_REQUEST, Json(response));
        }
    };

    match state.service.execute(request.clone()).await {
        Ok(response) => {
            state.logger.push(
                addr.ip().to_string(),
                request.entity.clone(),
                ErrorCode::None.as_u8(),
                None,
            );
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            let status = err.http_status();
            let error_code = err.code.as_u8();
            let error_message = err.message.clone();
            let response = error_response(request.clone(), err);

            state.logger.push(
                addr.ip().to_string(),
                request.entity,
                error_code,
                Some(error_message.clone()),
            );

            error!(
                "request failed: status={} error={} msg={}",
                status, error_code, error_message
            );

            (status, Json(response))
        }
    }
}

#[allow(dead_code)]
fn _unexpected_response_error() -> ApiError {
    ApiError::new(
        ErrorCode::ResponseBuild,
        json!({"msg": "unexpected response build error"}).to_string(),
    )
}
