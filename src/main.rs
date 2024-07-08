use std::fs;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

async fn retrieve(
    Path(id): Path<i32>,
    State(state): State<MyState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Paste>("SELECT * FROM pastes WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(todo) => Ok((StatusCode::OK, Json(todo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn add(
    State(state): State<MyState>,
    Form(data): Form<PasteNew>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Paste>(
        "INSERT INTO pastes (title, content) VALUES ($1, $2) RETURNING id, title, content",
    )
    .bind(&data.title)
    .bind(&data.content)
    .fetch_one(&state.pool)
    .await
    {
        Ok(todo) => Ok((StatusCode::CREATED, Json(todo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn send_html(file_name: String) -> Html<String> {
    let path = format!("{}/{}/{}", env!("CARGO_MANIFEST_DIR"), "src", file_name);
    let html = fs::read_to_string(path).expect("oof");
    Html(html)
}

#[derive(Clone)]
struct MyState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = MyState { pool };
    let router = Router::new()
        .route("/", get(|| send_html("index.html".to_string())))
        .route("/settings", get(|| send_html("settings.html".to_string())))
        .route("/paste", post(add))
        .route("/pastes/:id", get(retrieve))
        .with_state(state);

    Ok(router.into())
}

#[derive(Deserialize)]
struct PasteNew {
    pub title: String,
    pub content: String,
}

#[derive(Serialize, FromRow)]
struct Paste {
    pub id: i32,
    pub title: String,
    pub content: String,
}
