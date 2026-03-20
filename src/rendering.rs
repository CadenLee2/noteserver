const STYLE_RULES: &str = r#"
html {
    --bg: #e3e3e5;
    --text: #1e2128;
    --bg-pre: #c9c9cc;
    --text-pre: #3c3c42;
    --border: #babdc1;
    --link: #1165aa;
    --link-hover: #3f87c1;
}

.dark {
    --bg: #121216;
    --text: #d9dadd;
    --bg-pre: #26262d;
    --text-pre: #9c9ea8;
    --border: #3f4149;
    --link: #74b7db;
    --link-hover: #83c1e2;
}

body {
    font-size: 16px;
    background-color: var(--bg);
    color: var(--text);
    font-family: 'Lora', 'Arial';
    display: flex;
    justify-content: center;
    padding: 4px 16px;
}

body > div {
    width: 100%;
    max-width: 820px;
}

h1 {
    font-size: 26px;
}

h2 {
    font-size: 22px;
}

h3 {
    font-size: 18px;
}

h1, h2, h3 {
    margin-top: 22px;
    margin-bottom: 4px;
}

p {
    margin-block: 12px;
}

a, a:visited {
    color: var(--link);
}

a:hover {
    color: var(--link-hover);
    cursor: pointer;
}

pre, code {
    background-color: var(--bg-pre);
    color: var(--text-pre);
}

pre {
    border-radius: 4px;
    padding: 4px 8px;
}

code {
    border-radius: 4px;
    padding: 0px 2px;
}

.actions {
    color: #797b7d;
}

hr {
    border: solid 1px var(--border);
    margin-top: 40px;
}
"#;

fn front_matter(title: &str) -> String {
    format!(
        r#"
<head>
<title>{}</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Lora">
<style>
{}
</style>
</head>
"#,
        title, STYLE_RULES
    )
}

pub fn error_page(error: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
{}
<body>
    <div style="text-align:center;">
        <h1>404</h1>
        <p>{}</p>
    </div>
</body>
</html>
"#,
        front_matter("Notes"),
        error
    )
}

const MISC_DIR_ID: &str = "misc";

pub fn root() -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
{}
<body>
    <div>
        <h1>Notes</h1>
        <p>
            Self-hosted markdown notes. See the <a href="https://github.com/Cadecraft/noteserver">GitHub repository</a> for more information.
        </p>
    </div>
</body>
</html>
"#,
        front_matter("Notes")
    )
}

pub fn directory(
    dir: &str,
    note_titles: &[String],
    description: &Option<String>,
    darktheme: bool,
) -> String {
    let dir_descr_elem = match description {
        Some(d) => format!("<p>{}</p>", d),
        None => String::new(),
    };

    if dir == MISC_DIR_ID {
        return format!(
            r#"
<!DOCTYPE html>
<html class="{}">
{}
<body>
    <div>
        <h1>Notes directory: {}</h1>
        {}
        <p>
            <i>Cannot view all notes in this directory</i>
        </p>
    </div>
</body>
</html>
"#,
            if darktheme { "dark" } else { "" },
            front_matter(dir),
            dir,
            dir_descr_elem
        );
    }

    let note_list = note_titles
        .iter()
        .map(|n| format!("<li><a href=\"/{}/{}\">{}</a></li>", dir, n, n))
        .collect::<String>();

    format!(
        r#"
<!DOCTYPE html>
<html class="{}">
{}
<body>
    <div>
        <h1>Notes directory: {}</h1>
        {}
        <ul>
        {}
        </ul>
    </div>
</body>
</html>
"#,
        if darktheme { "dark" } else { "" },
        front_matter(dir),
        dir,
        dir_descr_elem,
        note_list
    )
}

pub fn note(dir: &str, note: &str, md_contents: &str, darktheme: bool) -> String {
    let md_as_html =
        markdown::to_html_with_options(md_contents, &markdown::Options::gfm()).unwrap();

    let note_title = format!("{} ({})", note, dir);

    let mut actions: Vec<String> = Vec::new();
    if dir != MISC_DIR_ID {
        actions.push(format!(
            "<a href=\"/{}\">Return to directory {}</a> • ",
            dir, dir
        ));
    }
    actions.push(format!("<a href=\"/{}/{}?raw=true\">Raw</a> • ", dir, note));
    let other_theme = if darktheme {
        ("light", "Light")
    } else {
        ("dark", "Dark")
    };
    actions.push(format!(
        "<a href=\"/{}/{}?theme={}\">{} mode</a>",
        dir, note, other_theme.0, other_theme.1
    ));

    let actions_str = actions.iter().map(|o| o.to_string()).collect::<String>();

    format!(
        r#"
<!DOCTYPE html>
<html class="{}">
{}
<body>
    <div>
        {}
        <hr id="end">
        <div class="actions">{}</div>
    </div>
</body>
</html>
"#,
        if darktheme { "dark" } else { "" },
        front_matter(&note_title),
        md_as_html,
        actions_str
    )
}
