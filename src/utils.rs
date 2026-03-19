use axum::http::StatusCode;
use axum::response::{Html, Response, IntoResponse};

use crate::rendering;

pub async fn create_dir(pool: &sqlx::PgPool, id: String, description: String) -> StatusCode {
    match sqlx::query!(
        r#"
        INSERT INTO directory
        (id, description)
        VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE
        SET description = EXCLUDED.description;
    "#,
        id,
        description,
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

// TODO: move to types
struct NoteMetadata {
    id: String,
}
struct NoteContents {
    md_contents: String,
}
struct DirectoryMetadata {
    description: Option<String>,
}

pub async fn get_dir(pool: &sqlx::PgPool, dir: String) -> Html<String> {
    let dir_data = match sqlx::query_as!(
        DirectoryMetadata,
        r#"
        SELECT description
        FROM directory
        WHERE id = $1
    "#,
        dir,
    )
    .fetch_all(pool)
    .await
    {
        // TODO: refactor this to "or error page"
        Ok(rows) => rows,
        Err(_) => return Html(rendering::error_page("Error fetching directory")),
    };

    if dir_data.is_empty() {
        return Html(rendering::error_page("Directory not found"));
    }

    let notes = match sqlx::query_as!(
        NoteMetadata,
        r#"
        SELECT id
        FROM note
        WHERE directory_id = $1;
    "#,
        dir,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return Html(rendering::error_page("Directory not found")),
    };

    // TODO: also get description from dir table

    let mut note_titles = notes.iter().map(|n| n.id.clone()).collect::<Vec<String>>();
    note_titles.sort();

    let description = &dir_data[0].description;
    Html(rendering::directory(&dir, &note_titles, description))
}

pub async fn get_note(pool: &sqlx::PgPool, dir: String, note: String, raw: bool) -> Response {
    let note_contents = match sqlx::query_as!(
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
    .await
    {
        Ok(rows) => rows,
        Err(_) => return Html(rendering::error_page("Note not found")).into_response(),
    };

    if raw {
        note_contents.md_contents.into_response()
    } else {
        Html(rendering::note(&dir, &note, &note_contents.md_contents)).into_response()
    }
}
