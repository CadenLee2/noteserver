pub const STYLE_RULES: &str = r#"
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
    font-size: 16px;
}

pre {
    border-radius: 4px;
    padding: 4px 8px;
    overflow: scroll;
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
