use crate::piping_server;
use std::collections::HashMap;
use std::ops::Deref;
use url::Url;

fn escape_html_attribute(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("'", "&apos;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

pub fn help(base_url: &Url) -> String {
    let version: &'static str = env!("CARGO_PKG_VERSION");
    return std::format!(
        r#"Help for Piping Server (Rust) {version}
(Repository: https://github.com/nwtgck/piping-server-rust)

======= Get  =======
curl {url}

======= Send =======
# Send a file
curl -T myfile {url}

# Send a text
echo 'hello!' | curl -T - {url}

# Send a directory (zip)
zip -q -r - ./mydir | curl -T - {url}

# Send a directory (tar.gz)
tar zfcp - ./mydir | curl -T - {url}

# Encryption
## Send
cat myfile | openssl aes-256-cbc | curl -T - {url}
## Get
curl {url} | openssl aes-256-cbc -d
"#,
        version = version,
        url = base_url.join("mypath").unwrap(),
    );
}

pub fn no_script_html(query_params: &HashMap<String, String>) -> String {
    let path_query_param_name = "path";
    let mode_query_param_name = "mode";
    let file_mode = "file";
    let text_mode = "text";

    let path = query_params
        .get(path_query_param_name)
        .map(|s| s.deref())
        .unwrap_or_else(|| "");

    let mode = query_params
        .get(mode_query_param_name)
        .map(|s| s.deref())
        .unwrap_or_else(|| "");

    let text_or_file_input = if mode == text_mode {
        std::format!(
            // language=html
            r#"<h3>Step 2: Input text</h3>
    <textarea name="input_text" cols="30" {disabled_or_row} placeholder="{placeholder}"></textarea>"#,
            disabled_or_row = if path.is_empty() {
                "disabled"
            } else {
                "rows='10'"
            },
            placeholder = if path.is_empty() {
                "Fill in the path above first"
            } else {
                ""
            },
        )
    } else {
        std::format!(
            // language=html
            r#"<h3>Step 2: Choose a file</h3>
    <input type="file" name="input_file" {disabled}>"#,
            disabled = if path.is_empty() { "disabled" } else { "" },
        )
    };

    return std::format!(
        // language=html
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <title>File transfer without JavaScript</title>
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <meta charset="UTF-8">
  <style>
    h3 {{
      margin-top: 2em;
      margin-bottom: 0.5em;
    }}
  </style>
</head>
<body>
  <h2>File transfer without JavaScript</h2>
  <form method="GET" action="{no_script_path}">
    <h3>Step 1: Specify path and mode</h3>
    <input name="{path_query_param_name}" value="{escaped_path}" size="30" placeholder='e.g. "abc123", "myimg.png"'>
    <input type="submit" value="Apply"><br>
    <input type="radio" name="{mode_query_param_name}" value="{file_mode}" {file_mode_input_checked}>File
    <input type="radio" name="{mode_query_param_name}" value="{text_mode}" {text_mode_input_checked}>Text<br>
  </form>
  <form method="POST" action="{escaped_path}" enctype="multipart/form-data">
    {text_or_file_input}
    <h3>Step 3: Send</h3>
    <input type="submit" value="Send" {disabled}>
  </form>
  <hr>
  Version {version} (Rust)<br>
  Piping Server:
  <a href="https://github.com/nwtgck/piping-server-rust">
    https://github.com/nwtgck/piping-server-rust
  </a><br>
  <a href=".">Top page</a><br>
</body>
</html>
"#,
        no_script_path = piping_server::reserved_paths::NO_SCRIPT,
        path_query_param_name = path_query_param_name,
        mode_query_param_name = mode_query_param_name,
        file_mode = file_mode,
        text_mode = text_mode,
        file_mode_input_checked = if mode == file_mode { "checked" } else { "" },
        text_mode_input_checked = if mode == text_mode { "checked" } else { "" },
        escaped_path = escape_html_attribute(path),
        text_or_file_input = text_or_file_input,
        disabled = if path.is_empty() { "disabled" } else { "" },
        version = env!("CARGO_PKG_VERSION"),
    );
}
