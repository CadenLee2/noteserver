fn front_matter(title: &str) -> String {
    format!(r#"
<head>
    <title>{}</title>
    <style>
        h1 {{
            font-size: 18px;
        }}
    </style>
</head>
"#, title)
}

pub fn error_page(error: &str) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
{}
<body>
    <h1>404 Error</h1>
    <p>{}</p>
</body>
</html>
"#, front_matter("Notes"), error)
}

pub fn directory(dir: &str, note_titles: &Vec<String>, description: &Option<String>) -> String {
    let note_list = note_titles
        .iter()
        .map(|n| format!("<li>{}</li>", n))
        .collect::<String>();

    let dir_descr_elem = match description {
        Some(d) => format!("<p>{}</p>", d),
        None => String::new(),
    };

    format!(
        r#"
<!DOCTYPE html>
<html>
{}
<body>
    <h1>Notes directory: {}</h1>
    {}
    <ul>
    {}
    </ul>
</body>
</html>
"#,
        front_matter(dir), dir, dir_descr_elem, note_list
    )
}

pub fn note(dir: &str, note: &str, md_contents: &str) -> String {
    // TODO: convert
    let md_as_html = md_contents;

    format!(
        r#"
<!DOCTYPE html>
<html>
{}
<body>
    {}
    <hr>
    <a href="/{}">Return to directory {}</a>
</body>
</html>
"#,
        front_matter(note), md_as_html, dir, dir
    )
}
