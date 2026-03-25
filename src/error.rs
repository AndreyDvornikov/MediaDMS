use axum::http::StatusCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    None = 0,
    DbConnection = 1,
    NotFound = 2,
    QueryBuild = 3,
    InvalidClientRequest = 4,
    ResponseBuild = 5,
    DbRead = 6,
    Unknown = 7,
}

impl ErrorCode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidClientRequest, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn from_repo_error(err: RepoError) -> Self {
        match err {
            RepoError::Connection(m) => Self::new(ErrorCode::DbConnection, m),
            RepoError::Read(m) => Self::new(ErrorCode::DbRead, m),
            RepoError::Query(m) => Self::new(ErrorCode::QueryBuild, m),
            RepoError::Unknown(m) => Self::new(ErrorCode::Unknown, m),
        }
    }

    pub fn http_status(&self) -> StatusCode {
        match self.code {
            ErrorCode::None => StatusCode::OK,
            ErrorCode::InvalidClientRequest => StatusCode::BAD_REQUEST,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::DbConnection | ErrorCode::DbRead => StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::QueryBuild => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::ResponseBuild | ErrorCode::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RepoError {
    Connection(String),
    Read(String),
    Query(String),
    Unknown(String),
}
