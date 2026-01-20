//! Sample RustPress App - Todo API
//!
//! A minimal example demonstrating:
//! - Router setup with Axum
//! - CRUD handlers
//! - Request validation
//! - Error handling
//! - Database queries with SQLx

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use validator::Validate;

// ============================================
// App State
// ============================================

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// ============================================
// Models
// ============================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTodo {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub completed: Option<bool>,
    pub limit: Option<i64>,
}

// ============================================
// Error Handling
// ============================================

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.leak()),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Internal(e.to_string()),
        }
    }
}

// ============================================
// Handlers
// ============================================

/// GET /todos - List all todos
pub async fn list_todos(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Todo>>, ApiError> {
    let limit = params.limit.unwrap_or(100).min(100);

    let todos = match params.completed {
        Some(completed) => {
            sqlx::query_as!(
                Todo,
                "SELECT id, title, completed FROM todos WHERE completed = $1 LIMIT $2",
                completed,
                limit
            )
            .fetch_all(&state.db)
            .await?
        }
        None => {
            sqlx::query_as!(
                Todo,
                "SELECT id, title, completed FROM todos LIMIT $1",
                limit
            )
            .fetch_all(&state.db)
            .await?
        }
    };

    Ok(Json(todos))
}

/// GET /todos/:id - Get single todo
pub async fn get_todo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, ApiError> {
    let todo = sqlx::query_as!(
        Todo,
        "SELECT id, title, completed FROM todos WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(todo))
}

/// POST /todos - Create todo
pub async fn create_todo(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), ApiError> {
    // Validate input
    input
        .validate()
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let todo = sqlx::query_as!(
        Todo,
        "INSERT INTO todos (title, completed) VALUES ($1, false) RETURNING id, title, completed",
        input.title
    )
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(todo)))
}

/// PUT /todos/:id - Update todo
pub async fn update_todo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTodo>,
) -> Result<Json<Todo>, ApiError> {
    let todo = sqlx::query_as!(
        Todo,
        r#"
        UPDATE todos SET
            title = COALESCE($1, title),
            completed = COALESCE($2, completed)
        WHERE id = $3
        RETURNING id, title, completed
        "#,
        input.title,
        input.completed,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(todo))
}

/// DELETE /todos/:id - Delete todo
pub async fn delete_todo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query!("DELETE FROM todos WHERE id = $1", id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Router Setup
// ============================================

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/:id", get(get_todo).put(update_todo).delete(delete_todo))
        .with_state(state)
}

// ============================================
// Main (for standalone testing)
// ============================================

#[tokio::main]
async fn main() {
    // In real app, get this from config
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/rustpress".into());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = Arc::new(AppState { db: pool });
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
