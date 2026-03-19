use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};

use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod config;
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

    // TODO: delete
    let app = Router::new()
        .route("/{dir}", post(post_dir))
        .route("/{dir}/{note}", post(post_note))
        .route("/{dir}", get(get_dir))
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

// TODO: secure routes with auth
async fn post_dir(
    Path(dir): Path<String>,
    State(state): State<Arc<AppState>>,
    body: String,
) -> StatusCode {
    utils::create_dir(&state.db_pool, dir, body).await
}

// TODO: secure routes with auth
async fn post_note(
    Path((dir, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    body: String,
) -> StatusCode {
    utils::create_note(&state.db_pool, dir, id, body).await
}

async fn get_dir(Path(dir): Path<String>, State(state): State<Arc<AppState>>) -> Html<String> {
    utils::get_dir(&state.db_pool, dir).await
}

async fn get_note(
    Path((dir, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    utils::get_note(&state.db_pool, dir, id).await
}
