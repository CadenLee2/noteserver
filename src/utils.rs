use axum::http::StatusCode;
use axum::response::Html;

pub async fn create_dir(pool: &sqlx::PgPool, id: String, description: String) -> StatusCode {
    match sqlx::query!(
        r#"
        INSERT INTO directory
        (id, description)
        VALUES ($1, $2);
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
        VALUES ($1, $2, $3);
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

// TODO: move constant to rendering folder and add og
const ERROR_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<body>
    <h1>404 Error</h1>
    <p>The resource could not be found</p>
</body>
</html> 
"#;

pub async fn get_dir(pool: &sqlx::PgPool, dir: String) -> Html<String> {
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
        Err(_) => return Html(ERROR_PAGE.to_string()),
    };

    // TODO: also get description from dir table

    let mut note_titles = notes.iter().map(|n| n.id.clone()).collect::<Vec<String>>();
    note_titles.sort();

    let note_list = note_titles
        .iter()
        .map(|n| format!("<li>{}</li>", n))
        .collect::<String>();

    let dir_template = format!(
        r#"
<!DOCTYPE html>
<html>
<body>
    <h1>Notes directory: {}</h1>
    <p>Notes include:</p>
    <ul>
    {}
    </ul>
</body>
</html> 
"#,
        dir, note_list
    );

    Html(dir_template)
}

pub async fn get_note(pool: &sqlx::PgPool, dir: String, note: String) -> Html<String> {
    let note_contents = match sqlx::query_as!(
        NoteContents,
        r#"
        SELECT md_contents
        FROM note
        WHERE (directory_id = $1);
    "#,
        dir
    )
    .fetch_one(pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return Html(ERROR_PAGE.to_string()),
    };

    // TODO: convert
    let md_as_html = note_contents.md_contents;

    let dir_template = format!(
        r#"
<!DOCTYPE html>
<html>
<body>
    <h1>{}</h1>
    {}
    <h1><a href="/{}">Return to directory {}</a></h1>
</body>
</html> 
"#,
        note, md_as_html, dir, dir
    );

    Html(dir_template)
}
