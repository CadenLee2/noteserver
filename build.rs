use sqlx::{
    Pool,
    postgres::{PgPoolOptions, Postgres},
};

use std::env;

pub async fn setup_schemas(pool: &Pool<Postgres>) {
    let mut transaction = pool.begin().await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS directory(
            id TEXT PRIMARY KEY,
            description TEXT,
            protected BOOLEAN
        );
        "#,
    )
    .execute(&mut *transaction)
    .await
    .expect("Failed to create directory table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS note(
            directory_id TEXT references directory(id),
            id TEXT NOT NULL,
            md_contents TEXT NOT NULL,
            PRIMARY KEY (directory_id, id)
        );
        "#,
    )
    .execute(&mut *transaction)
    .await
    .expect("Failed to create note table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS token(
            unlocks_directory_id TEXT references directory(id),
            tok TEXT PRIMARY KEY,
            created DATE NOT NULL DEFAULT CURRENT_DATE
        );
        "#,
    )
    .execute(&mut *transaction)
    .await
    .expect("Failed to create token table");

    transaction.commit().await.unwrap();
}

// Set up the database schema
#[tokio::main]
async fn main() {
    // Initialize env variables
    dotenvy::dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("BUILD: DATABASE_URL must be set");

    println!("Build: connecting to db...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Couldn't connect to the database");
    println!("Build: connected to db, setting up schemas");

    // Set up the database with schemas
    setup_schemas(&pool).await;
    println!("Build: finished setting up schemas");
}
