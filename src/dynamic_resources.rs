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

pub fn index() -> String {
    return std::format!(
        // language=html
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <title>Piping Server</title>
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <meta charset="UTF-8">
  <style>
    body {{
      font-family: "Avenir Next", Helvetica, Arial, sans-serif;
      font-size: 110%;
      margin: 1em;
    }}
    h3 {{
      margin-top: 2em;
      margin-bottom: 0.5em;
    }}
  </style>
</head>
<body>
<h1 style="display: inline">Piping Server</h1>
<span>(Rust) {version}</span>
<p>Streaming Data Transfer Server over HTTP/HTTPS</p>
<h3>Step 1: Choose a file or text</h3>
<input type="checkbox" id="text_mode" onchange="toggleInputMode()">: <b>Text mode</b><br><br>
<input type="file" id="file_input">
<textarea id="text_input" placeholder="Input text" cols="30" rows="10" style="display: none"></textarea>
<br>
<h3>Step 2: Write your secret path</h3>
(e.g. "abcd1234", "mysecret.png")<br>
<input id="secret_path" placeholder="Secret path" size="50"><br>
<h3>Step 3: Click the send button</h3>
<button onclick="send()">Send</button><br>
<progress id="progress_bar" value="0" max="100" style="display: none"></progress><br>
<div id="message"></div>
<hr>
<a href="https://piping-ui.org">Piping UI for Web</a><br>
<a href="{no_script_path}">Transfer without JavaScript</a><br>
<a href="https://github.com/nwtgck/piping-server-rust">Source code on GitHub</a><br>
<script>
  // Toggle input mode: file or text
  var toggleInputMode = (function () {{
    var activeInput      = window.file_input;
    var deactivatedInput = window.text_input;
    // Set inputs' functionality and visibility
    function setInputs() {{
      activeInput.removeAttribute("disabled");
      activeInput.style.removeProperty("display");
      deactivatedInput.setAttribute("disabled", "");
      deactivatedInput.style.display = "none";
    }}
    setInputs();
    // Body of toggleInputMode
    function toggle() {{
      // Swap inputs
      var tmpInput     = activeInput;
      activeInput      = deactivatedInput;
      deactivatedInput = tmpInput;
      setInputs();
    }}
    return toggle;
  }})();
  function setMessage(msg) {{
    window.message.innerText = msg;
  }}
  function setProgress(loaded, total) {{
    var progress = (total === 0) ? 0 : loaded / total * 100;
    window.progress_bar.value = progress;
    setMessage(loaded + "B (" + progress.toFixed(2) + "%)");
  }}
  function hideProgress() {{
    window.progress_bar.style.display = "none";
  }}
  function send() {{
    // Select body (text or file)
    var body = window.text_mode.checked ? window.text_input.value : window.file_input.files[0];
    // Send
    var xhr = new XMLHttpRequest();
    var path = location.href.replace(/\/$/, '') + "/" + window.secret_path.value;
    xhr.open("POST", path, true);
    // If file has no type
    if (!window.text_mode.checked && body.type === "") {{
      xhr.setRequestHeader("Content-Type", "application/octet-stream");
    }}
    // Update progress bar
    xhr.upload.onprogress = function (e) {{
      setProgress(e.loaded, e.total);
    }};
    xhr.upload.onload = function (e) {{
      // Send finished
      if (xhr.status === 200) {{
        setProgress(e.loaded, e.total);
      }}
    }};
    xhr.onload = function () {{
      // Status code error
      if (xhr.status !== 200) {{
        setMessage(xhr.responseText);
        hideProgress();
      }}
    }};
    xhr.onerror = function () {{
      setMessage("Upload error");
      hideProgress();
    }};
    xhr.send(body);
    // Show progress bar
    window.progress_bar.style.removeProperty("display");
  }}
</script>
</body>
</html>
"#,
        version = env!("CARGO_PKG_VERSION"),
        no_script_path = &piping_server::reserved_paths::NO_SCRIPT[1..],
    );
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

pub fn no_script_html(query_params: &HashMap<String, String>, style_nonce: &str) -> String {
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
        .unwrap_or_else(|| file_mode);

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

    let escaped_path = escape_html_attribute(path);

    return std::format!(
        // language=html
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <title>File transfer without JavaScript</title>
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <meta charset="UTF-8">
  <style nonce="{style_nonce}">
    body {{
      font-family: sans-serif;
      font-size: 110%;
    }}
    h3 {{
      margin-top: 2em;
      margin-bottom: 0.5em;
    }}
  </style>
</head>
<body>
  <h2>File transfer without JavaScript</h2>
  <form method="GET">
    <h3>Step 1: Specify path and mode</h3>
    <input name="{path_query_param_name}" value="{escaped_path}" size="30" placeholder='e.g. "abc123", "myimg.png"'>
    <input type="submit" value="Apply"><br>
    <input type="radio" name="{mode_query_param_name}" value="{file_mode}" {file_mode_input_checked}>File
    <input type="radio" name="{mode_query_param_name}" value="{text_mode}" {text_mode_input_checked}>Text<br>
  </form>
  <form method="POST" {post_action} enctype="multipart/form-data">
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
        style_nonce = style_nonce,
        path_query_param_name = path_query_param_name,
        mode_query_param_name = mode_query_param_name,
        file_mode = file_mode,
        text_mode = text_mode,
        file_mode_input_checked = if mode == file_mode { "checked" } else { "" },
        text_mode_input_checked = if mode == text_mode { "checked" } else { "" },
        escaped_path = escaped_path,
        post_action = if path.is_empty() {
            "".to_string()
        } else {
            std::format!(r#"action="{escaped_path}""#, escaped_path = escaped_path)
        },
        text_or_file_input = text_or_file_input,
        disabled = if path.is_empty() { "disabled" } else { "" },
        version = env!("CARGO_PKG_VERSION"),
    );
}
