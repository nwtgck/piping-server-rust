use crate::piping_server;

fn escape_html_attribute(s: &str) -> String {
    return s
        .replace("&", "&amp;")
        .replace("'", "&apos;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;");
}

pub fn no_script_html(path: &String) -> String {
    // language=html
    return std::format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <title>File transfer without JavaScript</title>
  <meta name="viewport" content="width=device-width,initial-scale=1">
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
    <h3>Step 1: Specify path</h3>
    <input name="{path_query}" value="{escaped_path}">
    <input type="submit" value="Apply">
  </form>
  <form method="POST" action="{escaped_path}" enctype="multipart/form-data">
    <h3>Step 2: Choose a file</h3>
    <input type="file" name="input_file" {disabled}>
    <h3>Step 3: Send</h3>
    <input type="submit" value="Send" {disabled}>
  </form>
  <hr>
  Piping Server:
  <a href="https://github.com/nwtgck/piping-server-rust">
    https://github.com/nwtgck/piping-server-rust
  </a><br>
</body>
</html>
"#,
        no_script_path = piping_server::reserved_paths::NO_SCRIPT,
        path_query = piping_server::NO_SCRIPT_PATH_QUERY_PARAMETER_NAME,
        escaped_path = escape_html_attribute(path),
        disabled = if path.is_empty() { "disabled" } else { "" },
    );
}
