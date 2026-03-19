const STYLE_RULES: &str = r#"
body {
    font-size: 16px;
    background-color: #e3e3e5;
    color: #1e2128;
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
    color: #1165aa;
}

a:hover {
    color: #3f87c1;
    cursor: pointer;
}

hr {
    border: solid 1px #babdc1;
    margin-top: 40px;
}
"#;

fn front_matter(title: &str) -> String {
    format!(r#"
<head>
<title>{}</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Lora">
<style>
{}
</style>
</head>
"#, title, STYLE_RULES)
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
    let dir_descr_elem = match description {
        Some(d) => format!("<p>{}</p>", d),
        None => String::new(),
    };

    const MISC_DIR_ID: &str = "misc";

    if dir == MISC_DIR_ID {
        return format!(
            r#"
<!DOCTYPE html>
<html>
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
            front_matter(dir), dir, dir_descr_elem
        );
    }

    let note_list = note_titles
        .iter()
        .map(|n| format!("<li><a href=\"/{}/{}\">{}</a></li>", dir, n, n))
        .collect::<String>();

    format!(
        r#"
<!DOCTYPE html>
<html>
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
        front_matter(dir), dir, dir_descr_elem, note_list
    )
}

pub fn note(dir: &str, note: &str, md_contents: &str) -> String {
    let md_as_html = markdown::to_html(md_contents);

    let note_title = format!("{} ({})", note, dir);

    format!(
        r#"
<!DOCTYPE html>
<html>
{}
<body>
    <div>
        {}
        <hr id="end">
        <a id="return-to-dir" href="/{}">Return to directory {}</a>
    </div>
</body>
</html>
"#,
        front_matter(&note_title), md_as_html, dir, dir
    )
}
