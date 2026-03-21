use crate::styling::STYLE_RULES;

fn front_matter(title: &str, descr: Option<&str>) -> String {
    let descr_elem = match descr {
        Some(d) => format!("<meta property=\"og:description\" content=\"{}\" />", d),
        None => String::new(),
    };

    format!(
        r#"
<head>
<title>{}</title>
<meta property="og:title" content="{}" />
{}
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Lora">
<style>
{}
</style>
</head>
"#,
        title, title, descr_elem, STYLE_RULES
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
        front_matter("Notes", None),
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
        front_matter("Notes", Some("Self-hosted markdown notes"))
    )
}

pub fn directory(
    dir: &str,
    note_titles: &[String],
    description: Option<&str>,
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
            front_matter(dir, description),
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
        front_matter(dir, description),
        dir,
        dir_descr_elem,
        note_list
    )
}

fn to_valid_descr_char(c: char) -> char {
    if c == '"' {
        '\''
    } else if c.is_whitespace() || c == '>' || c == '<' || c == '&' {
        ' '
    } else {
        c
    }
}

fn descr_from_contents(md_contents: &str) -> String {
    let start = md_contents
        .chars()
        .take(60)
        .map(to_valid_descr_char)
        .collect::<String>();
    if start.chars().count() < md_contents.chars().count() {
        start + "..."
    } else {
        start
    }
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

    let note_descr = descr_from_contents(md_contents);

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
        front_matter(&note_title, Some(&note_descr)),
        md_as_html,
        actions_str
    )
}
