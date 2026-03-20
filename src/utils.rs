use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::rendering;

pub async fn create_dir(
    pool: &sqlx::PgPool,
    id: String,
    description: String,
    protected: bool,
) -> StatusCode {
    match sqlx::query!(
        r#"
        INSERT INTO directory
        (id, description, protected)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE
        SET description = EXCLUDED.description, protected = EXCLUDED.protected;
    "#,
        id,
        description,
        protected,
    )
    .fetch_all(pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn create_note(
    pool: &sqlx::PgPool,
    dir: String,
    id: String,
    contents: String,
) -> StatusCode {
    match sqlx::query!(
        r#"
        INSERT INTO note
        (directory_id, id, md_contents)
        VALUES ($1, $2, $3)
        ON CONFLICT (directory_id, id) DO UPDATE
        SET md_contents = EXCLUDED.md_contents;
    "#,
        dir,
        id,
        contents,
    )
    .fetch_all(pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn create_token(pool: &sqlx::PgPool, token: String, dir: String) -> StatusCode {
    match sqlx::query!(
        r#"
        INSERT INTO token
        (unlocks_directory_id, tok)
        VALUES ($1, $2);
    "#,
        dir,
        token,
    )
    .fetch_all(pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn delete_token(pool: &sqlx::PgPool, token: String) -> StatusCode {
    match sqlx::query!(
        r#"
        DELETE FROM token
        WHERE tok = $1;
    "#,
        token,
    )
    .fetch_all(pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// TODO: move to types
struct NoteMetadata {
    id: String,
}
struct NoteContents {
    md_contents: String,
}
struct DirectoryMetadata {
    description: Option<String>,
    protected: Option<bool>,
}

async fn is_valid_token(pool: &sqlx::PgPool, dir: &str, token: &str) -> bool {
    let token_info = match sqlx::query!(
        r#"
        SELECT FROM token
        WHERE tok = $1 AND unlocks_directory_id = $2
    "#,
        token,
        dir,
    )
    .fetch_all(pool)
    .await
    {
        // TODO: refactor this to "or error page"
        Ok(rows) => rows,
        Err(_) => return false,
    };
    !token_info.is_empty()
}

async fn get_dir_metadata(pool: &sqlx::PgPool, dir: &str) -> Option<DirectoryMetadata> {
    sqlx::query_as!(
        DirectoryMetadata,
        r#"
        SELECT description, protected
        FROM directory
        WHERE id = $1
    "#,
        dir,
    )
    .fetch_one(pool)
    .await
    .ok()
}

pub async fn get_dir(
    pool: &sqlx::PgPool,
    dir: String,
    token: Option<String>,
    darktheme: bool,
) -> Html<String> {
    let (dir_metadata_res, note_query_res) = tokio::join!(
        get_dir_metadata(pool, &dir),
        sqlx::query_as!(
            NoteMetadata,
            r#"
            SELECT id
            FROM note
            WHERE directory_id = $1;
        "#,
            dir,
        )
        .fetch_all(pool)
    );

    let target_dir = match dir_metadata_res {
        Some(res) => res,
        None => return Html(rendering::error_page("Directory not found")),
    };

    let valid_auth = if target_dir.protected.unwrap_or(false) {
        match token {
            Some(tok) => is_valid_token(pool, &dir, &tok).await,
            None => false,
        }
    } else {
        true
    };

    if !valid_auth {
        return Html(rendering::error_page("Directory not found"));
    }

    let notes = match note_query_res {
        Ok(rows) => rows,
        Err(_) => return Html(rendering::error_page("Directory not found")),
    };

    let mut note_titles = notes.iter().map(|n| n.id.clone()).collect::<Vec<String>>();
    note_titles.sort();

    let description = &target_dir.description;
    Html(rendering::directory(
        &dir,
        &note_titles,
        description,
        darktheme,
    ))
}

pub async fn get_note(
    pool: &sqlx::PgPool,
    dir: String,
    note: String,
    raw: bool,
    token: Option<String>,
    darktheme: bool,
) -> Response {
    let (dir_metadata_res, note_query_res) = tokio::join!(
        get_dir_metadata(pool, &dir),
        sqlx::query_as!(
            NoteContents,
            r#"
            SELECT md_contents
            FROM note
            WHERE directory_id = $1 AND id = $2;
        "#,
            dir,
            note,
        )
        .fetch_one(pool)
    );

    let target_dir = match dir_metadata_res {
        Some(res) => res,
        None => return Html(rendering::error_page("Note not found")).into_response(),
    };

    let valid_auth = if target_dir.protected.unwrap_or(false) {
        match token {
            Some(tok) => is_valid_token(pool, &dir, &tok).await,
            None => false,
        }
    } else {
        true
    };

    if !valid_auth {
        return Html(rendering::error_page("Note not found")).into_response();
    }

    let note_contents = match note_query_res {
        Ok(rows) => rows,
        Err(_) => return Html(rendering::error_page("Note not found")).into_response(),
    };

    if raw {
        note_contents.md_contents.into_response()
    } else {
        Html(rendering::note(
            &dir,
            &note,
            &note_contents.md_contents,
            darktheme,
        ))
        .into_response()
    }
}
