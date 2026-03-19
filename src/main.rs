use axum::{
    Router,
    extract::{Path, Query, State},
    http::{
        StatusCode,
        header::{self, HeaderMap},
    },
    response::{Html, Response},
    routing::{get, post, delete},
};

use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod config;
mod rendering;
mod utils;

use noteserver::auth;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = &config::get_config()["DATABASE_URL"];
    let port = &config::get_config()["PORT"];

    println!("Connecting to db...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await
        .expect("Couldn't connect to the database");
    println!("Connected to db");

    let state = Arc::new(AppState { db_pool: pool });

    // TODO: delete
    let app = Router::new()
        .route("/token/{tok}", post(post_token))
        .route("/token/{tok}", delete(delete_token))
        .route("/{dir}", post(post_dir))
        .route("/{dir}/{note}", post(post_note))
        .route("/{dir}", get(get_dir))
        .route("/{dir}/", get(get_dir))
        .route("/{dir}/{note}", get(get_note))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!(
        "noteserver v{} is listening on {}",
        option_env!("CARGO_PKG_VERSION").unwrap_or("?"),
        addr
    );

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn valid_auth(headers: &HeaderMap) -> bool {
    match headers.get(header::AUTHORIZATION).cloned() {
        Some(val) => match val.to_str() {
            Ok(pw) => auth::is_authorized(pw),
            _ => false,
        },
        None => false,
    }
}

#[derive(Deserialize)]
struct DirDetails {
    protected: Option<bool>
}

// TODO: secure routes with auth
async fn post_dir(
    Query(query): Query<DirDetails>,
    Path(dir): Path<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> StatusCode {
    if !valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    let protected = query.protected.unwrap_or(false);
    utils::create_dir(&state.db_pool, dir, body, protected).await
}

// TODO: secure routes with auth
async fn post_note(
    Path((dir, id)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> StatusCode {
    if !valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    utils::create_note(&state.db_pool, dir, id, body).await
}

#[derive(Deserialize)]
struct TokenDetails {
    directory: String,
}

async fn post_token(
    Query(query): Query<TokenDetails>,
    Path(token): Path<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    if !valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    utils::create_token(&state.db_pool, token, query.directory).await
}

async fn delete_token(
    Path(token): Path<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    if !valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    utils::delete_token(&state.db_pool, token).await
}

async fn get_dir(Path(dir): Path<String>, State(state): State<Arc<AppState>>) -> Html<String> {
    utils::get_dir(&state.db_pool, dir).await
}

#[derive(Deserialize)]
struct NoteRaw {
    raw: Option<bool>,
}

async fn get_note(
    Query(query): Query<NoteRaw>,
    Path((dir, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let raw = query.raw.unwrap_or(false);
    utils::get_note(&state.db_pool, dir, id, raw).await
}
