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

pub async fn delete_dir(pool: &sqlx::PgPool, dir: String) -> StatusCode {
    match sqlx::query!(
        r#"
        DELETE FROM directory
        WHERE id = $1;
    "#,
        dir,
    )
    .fetch_all(pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn delete_note(pool: &sqlx::PgPool, dir: String, id: String) -> StatusCode {
    match sqlx::query!(
        r#"
        DELETE FROM note
        WHERE directory_id = $1 AND id = $2;
    "#,
        dir,
        id,
    )
    .fetch_all(pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

struct DirectoryAdminResponse {
    id: String,
    protected: Option<bool>,
    notes_in_dir: Option<i64>,
}

struct Token {
    tok: String,
    unlocks_directory_id: Option<String>,
}

pub async fn get_all_admin(pool: &sqlx::PgPool) -> String {
    let (dir_resp, token_resp) = tokio::join!(
        sqlx::query_as!(
            DirectoryAdminResponse,
            r#"
        SELECT directory.id, protected, COUNT(note.id) AS notes_in_dir
        FROM directory
        LEFT JOIN note ON note.directory_id = directory.id
        GROUP BY directory.id;
    "#
        )
        .fetch_all(pool),
        sqlx::query_as!(
            Token,
            r#"
        SELECT tok, unlocks_directory_id
        FROM token;
    "#
        )
        .fetch_all(pool)
    );

    let dirs = match dir_resp {
        Ok(rows) => rows,
        Err(_) => return String::from("Error fetching dirs"),
    };

    let tokens = match token_resp {
        Ok(rows) => rows,
        Err(_) => return String::from("Error fetching tokens"),
    };

    let dirs_disp = dirs
        .iter()
        .map(|r| {
            format!(
                "/{} | {} notes | {}\n",
                r.id,
                r.notes_in_dir.unwrap_or(0),
                if r.protected.unwrap_or(false) {
                    "protected"
                } else {
                    "-"
                },
            )
        })
        .collect::<String>();

    let tokens_disp = tokens
        .iter()
        .map(|t| {
            format!(
                "{} | unlocks /{}\n",
                t.tok,
                t.unlocks_directory_id
                    .clone()
                    .unwrap_or(String::from("(none)"))
            )
        })
        .collect::<String>();

    format!("Directories:\n{}\n\nTokens:\n{}", dirs_disp, tokens_disp)
}

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

pub fn get_root() -> Html<String> {
    Html(rendering::root())
}

pub async fn get_dir(
    pool: &sqlx::PgPool,
    dir: String,
    token: Option<String>,
    darktheme: bool,
) -> Html<String> {
    const DIR_ERROR: &str = "Directory not found";
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
        None => return Html(rendering::error_page(DIR_ERROR)),
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
        return Html(rendering::error_page(DIR_ERROR));
    }

    let notes = match note_query_res {
        Ok(rows) => rows,
        Err(_) => return Html(rendering::error_page(DIR_ERROR)),
    };

    let mut note_titles = notes.iter().map(|n| n.id.clone()).collect::<Vec<String>>();
    note_titles.sort();

    let description = target_dir.description.as_deref();
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
    const NOTE_ERROR: &str = "Note not found";

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
        None => return Html(rendering::error_page(NOTE_ERROR)).into_response(),
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
        return Html(rendering::error_page(NOTE_ERROR)).into_response();
    }

    let note_contents = match note_query_res {
        Ok(rows) => rows,
        Err(_) => return Html(rendering::error_page(NOTE_ERROR)).into_response(),
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
