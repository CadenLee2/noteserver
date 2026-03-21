use axum::{
    Router,
    extract::{Path, Query, State},
    http::{StatusCode, header::HeaderMap},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};

use axum_extra::extract::cookie::CookieJar;

use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod actions;
mod config;
mod rendering;
mod styling;
mod utils;

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

    let app = Router::new()
        .route("/token/{tok}", post(post_token))
        .route("/token/{tok}", delete(delete_token))
        .route("/all", get(get_all))
        .route("/{dir}", post(post_dir))
        .route("/{dir}/{note}", post(post_note))
        .route("/{dir}", delete(delete_dir))
        .route("/{dir}/{note}", delete(delete_note))
        .route("/", get(get_root))
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

#[derive(Deserialize)]
struct DirDetails {
    protected: Option<bool>,
}

async fn post_dir(
    Query(query): Query<DirDetails>,
    Path(dir): Path<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> StatusCode {
    if !utils::valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    let protected = query.protected.unwrap_or(false);
    actions::create_dir(&state.db_pool, dir, body, protected).await
}

async fn post_note(
    Path((dir, id)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> StatusCode {
    if !utils::valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    actions::create_note(&state.db_pool, dir, id, body).await
}

#[derive(Deserialize)]
struct PostTokenQuery {
    directory: String,
}

async fn post_token(
    Query(query): Query<PostTokenQuery>,
    Path(token): Path<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    if !utils::valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    actions::create_token(&state.db_pool, token, query.directory).await
}

async fn delete_token(
    Path(token): Path<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    if !utils::valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    actions::delete_token(&state.db_pool, token).await
}

async fn delete_dir(
    Path(dir): Path<String>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    if !utils::valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    actions::delete_dir(&state.db_pool, dir).await
}

async fn delete_note(
    Path((dir, id)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    if !utils::valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    actions::delete_note(&state.db_pool, dir, id).await
}

async fn get_all(headers: HeaderMap, State(state): State<Arc<AppState>>) -> Response {
    if !utils::valid_auth(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    actions::get_all_admin(&state.db_pool).await.into_response()
}

async fn get_root() -> Response {
    actions::get_root().into_response()
}

#[derive(Deserialize)]
struct GetDirQuery {
    tok: Option<String>,
}

async fn get_dir(
    Query(query): Query<GetDirQuery>,
    Path(dir): Path<String>,
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
) -> Response {
    let token_cookie_name = utils::get_token_cookie_name(&dir);
    if let Some(tok) = &query.tok {
        let new_jar = utils::refresh_cookie_expiry(&jar);
        return (
            StatusCode::TEMPORARY_REDIRECT,
            utils::make_redirect_headers(format!("/{}", dir)),
            new_jar.add(utils::make_expiring_cookie(&token_cookie_name, tok)),
        )
            .into_response();
    };
    let tok = utils::get_cookie_from_jar(&jar, &token_cookie_name);
    let use_dark = utils::is_dark_theme(&jar);
    actions::get_dir(&state.db_pool, dir, tok, use_dark)
        .await
        .into_response()
}

#[derive(Deserialize)]
struct GetNoteQuery {
    raw: Option<bool>,
    theme: Option<String>,
}

async fn get_note(
    Query(query): Query<GetNoteQuery>,
    Path((dir, id)): Path<(String, String)>,
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
) -> Response {
    let raw = query.raw.unwrap_or(false);
    if let Some(theme) = &query.theme {
        let new_jar = utils::refresh_cookie_expiry(&jar);
        return (
            StatusCode::TEMPORARY_REDIRECT,
            utils::make_redirect_headers(format!("/{}/{}", dir, id)),
            new_jar.add(utils::make_expiring_cookie("theme", theme)),
        )
            .into_response();
    };
    let token_cookie_name = utils::get_token_cookie_name(&dir);
    let tok = utils::get_cookie_from_jar(&jar, &token_cookie_name);
    let use_dark = utils::is_dark_theme(&jar);
    actions::get_note(&state.db_pool, dir, id, raw, tok, use_dark).await
}
