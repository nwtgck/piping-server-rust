use futures::channel::oneshot;
use hyper::service::{make_service_fn, service_fn};
use hyper::Client;
use hyper::Server;
use regex::Regex;
use specit::tokio_it as it;
use std::convert::Infallible;

use piping_server::piping_server::PipingServer;
use piping_server::req_res_handler::req_res_handler;
use std::net::SocketAddr;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn get_header_value<'a>(
    headers: &'a hyper::header::HeaderMap,
    key: &'static str,
) -> Option<&'a str> {
    headers.get(key).map(|v| v.to_str().unwrap())
}

// Read all bytes from body
async fn read_all_body(mut body: hyper::body::Body) -> Vec<u8> {
    use futures::stream::StreamExt;

    let mut all_bytes: Vec<u8> = Vec::new();
    loop {
        if let Some(Ok(bytes)) = body.next().await {
            all_bytes.append(&mut bytes.to_vec());
        } else {
            break;
        }
    }
    all_bytes
}

struct Serve {
    addr: SocketAddr,
    shutdown_tx: oneshot::Sender<()>,
    shutdown_finished_rx: oneshot::Receiver<()>,
}

// Serve Piping Server on available port
async fn serve() -> Serve {
    let piping_server = PipingServer::new();

    let (addr_tx, addr_rx) = oneshot::channel::<SocketAddr>();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (shutdown_finished_tx, shutdown_finished_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let http_svc = make_service_fn(|_| {
            let piping_server = piping_server.clone();
            let handler = req_res_handler(move |req, res_sender| {
                piping_server.handler(false, req, res_sender)
            });
            futures::future::ok::<_, Infallible>(service_fn(handler))
        });
        let http_server = Server::bind(&([127, 0, 0, 1], 0).into()).serve(http_svc);
        addr_tx
            .send(http_server.local_addr())
            .expect("server address send failed");

        http_server
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("failed to shutdown in server-side");
        shutdown_finished_tx.send(()).unwrap();
    });

    let addr = addr_rx.await.expect("failed to get addr");

    Serve {
        addr,
        shutdown_tx,
        shutdown_finished_rx,
    }
}

impl Serve {
    async fn shutdown(self) -> Result<(), BoxError> {
        self.shutdown_tx
            .send(())
            .expect("failed to shutdown in client-side");
        self.shutdown_finished_rx.await?;
        Ok(())
    }
}

#[it("should return index page")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, body) = res.into_parts();

    let body_string = String::from_utf8(read_all_body(body).await)?;
    // Body should contain "Piping"
    assert!(body_string.contains("Piping"));
    // Body should specify charset
    assert!(body_string
        .to_lowercase()
        .contains(r#"<meta charset="utf-8">"#));

    // Content-Type is "text/html"
    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/html")
    );

    serve.shutdown().await?;
    Ok(())
}

#[it("should return noscript page")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/noscript?path=mypath", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, body) = res.into_parts();

    let body_string = String::from_utf8(read_all_body(body).await)?;
    // Body should contain "Piping"
    assert!(body_string.contains("action=\"mypath\""));
    // Body should specify charset
    assert!(body_string
        .to_lowercase()
        .contains(r#"<meta charset="utf-8">"#));

    // Content-Type is "text/html"
    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/html")
    );

    // Should disable JavaScript and allow CSS with nonce
    assert!(Regex::new(r"^default-src 'none'; style-src 'nonce-.+'$")
        .unwrap()
        .is_match(&get_header_value(&parts.headers, "content-security-policy").unwrap()));

    serve.shutdown().await?;
    Ok(())
}

#[it("should return version page")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/version", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, body) = res.into_parts();

    // Body should contains version
    let body_string = String::from_utf8(read_all_body(body).await)?;
    assert!(body_string.contains(env!("CARGO_PKG_VERSION")));
    // Body should contains case-insensitive "rust"
    assert!(body_string.to_lowercase().contains("rust"));

    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-origin"),
        Some("*")
    );

    serve.shutdown().await?;
    Ok(())
}

#[it("should return help page")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/help", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, _) = res.into_parts();

    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-origin"),
        Some("*")
    );

    serve.shutdown().await?;
    Ok(())
}

#[it("should handle /favicon.ico")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/favicon.ico", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, body) = res.into_parts();

    // Should be no body
    let body_len: usize = read_all_body(body).await.len();
    assert_eq!(body_len, 0);

    let status: http::StatusCode = parts.status;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    serve.shutdown().await?;
    Ok(())
}

#[it("should handle /robots.txt")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/robots.txt", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, body) = res.into_parts();

    // Should be no body
    let body_len: usize = read_all_body(body).await.len();
    assert_eq!(body_len, 0);

    assert_eq!(parts.status, http::StatusCode::NOT_FOUND);

    serve.shutdown().await?;
    Ok(())
}

#[it("should return a HEAD response with the same headers as GET response in the reserved paths")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    fn normalize_headers(headers: &mut http::header::HeaderMap<http::header::HeaderValue>) {
        headers.remove("date");
        headers.remove("content-security-policy");
    }

    let client = Client::new();
    for reserved_path in piping_server::piping_server::reserved_paths::VALUES {
        let uri = format!("http://{}{}", serve.addr, reserved_path).parse::<http::Uri>()?;

        let get_req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(uri.clone())
            .body(hyper::Body::empty())?;
        let get_res = client.request(get_req).await?;
        let (mut get_parts, _) = get_res.into_parts();
        normalize_headers(&mut get_parts.headers);

        let head_req = hyper::Request::builder()
            .method(hyper::Method::HEAD)
            .uri(uri.clone())
            .body(hyper::Body::empty())?;
        let head_res = client.request(head_req).await?;
        let (mut head_parts, _) = head_res.into_parts();
        normalize_headers(&mut head_parts.headers);

        assert_eq!(head_parts.status, get_parts.status);
        assert_eq!(head_parts.headers, get_parts.headers);
    }

    serve.shutdown().await?;
    Ok(())
}

#[it("should support preflight request")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/mypath", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::OPTIONS)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, _body) = res.into_parts();

    assert_eq!(parts.status, http::StatusCode::OK);

    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-origin"),
        Some("*")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-methods"),
        Some("GET, HEAD, POST, PUT, OPTIONS")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-headers")
            .unwrap()
            .to_lowercase(),
        "content-type, content-disposition, x-piping".to_owned()
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-max-age"),
        Some("86400")
    );

    serve.shutdown().await?;
    Ok(())
}

#[it("should reject Service Worker registration request")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/mysw.js", serve.addr).parse::<http::Uri>()?;

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .header("service-worker", "script")
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let res = client.request(get_req).await?;

    let (parts, _body) = res.into_parts();

    assert_eq!(parts.status, http::StatusCode::BAD_REQUEST);
    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-origin"),
        Some("*")
    );

    serve.shutdown().await?;
    Ok(())
}

#[it("should reject POST and PUT with Content-Range")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/mypath", serve.addr).parse::<http::Uri>()?;

    for method in [hyper::Method::POST, hyper::Method::PUT] {
        let get_req = hyper::Request::builder()
            .method(method)
            .uri(uri.clone())
            .header("Content-Range", "bytes 2-6/100")
            .body(hyper::Body::from("hello"))?;
        let client = Client::new();
        let res = client.request(get_req).await?;
        let (parts, _body) = res.into_parts();

        assert_eq!(parts.status, http::StatusCode::BAD_REQUEST);
        assert_eq!(
            get_header_value(&parts.headers, "content-type"),
            Some("text/plain")
        );
        assert_eq!(
            get_header_value(&parts.headers, "access-control-allow-origin"),
            Some("*")
        );
    }

    serve.shutdown().await?;
    Ok(())
}

#[it("should handle connection (sender: O, receiver: O)")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/mypath", serve.addr).parse::<http::Uri>()?;

    let send_body_str = "this is a content";
    let send_req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header("Content-Type", "text/plain")
        .uri(uri.clone())
        .body(hyper::Body::from(send_body_str))?;

    let client = Client::new();
    let send_res = client.request(send_req).await?;
    let (send_res_parts, _send_res_body) = send_res.into_parts();
    assert_eq!(send_res_parts.status, http::StatusCode::OK);
    assert_eq!(
        get_header_value(&send_res_parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&send_res_parts.headers, "access-control-allow-origin"),
        Some("*")
    );

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let (parts, body) = client.request(get_req).await?.into_parts();

    let all_bytes: Vec<u8> = read_all_body(body).await;

    let expect = send_body_str.to_owned().into_bytes();
    assert_eq!(all_bytes, expect);

    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&parts.headers, "content-length"),
        Some(send_body_str.len().to_string().as_str())
    );
    assert_eq!(
        get_header_value(&parts.headers, "content-disposition"),
        None
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-origin"),
        Some("*")
    );
    assert_eq!(
        get_header_value(&parts.headers, "x-robots-tag"),
        Some("none")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-expose-headers"),
        None,
    );

    serve.shutdown_tx.send(()).expect("shutdown failed");
    Ok(())
}

#[it("should handle connection (receiver: O, sender: O)")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/mypath", serve.addr).parse::<http::Uri>()?;

    let get_res_join_handle = tokio::spawn({
        let uri = uri.clone();
        async {
            let get_req = hyper::Request::builder()
                .method(hyper::Method::GET)
                .uri(uri)
                .body(hyper::Body::empty())?;
            let client = Client::new();
            let get_res = client.request(get_req).await?;
            Ok::<_, BoxError>(get_res)
        }
    });

    let send_body_str = "this is a content";
    let send_req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header("Content-Type", "text/plain")
        .uri(uri.clone())
        .body(hyper::Body::from(send_body_str))?;

    let client = Client::new();
    let send_res = client.request(send_req).await?;
    let (send_res_parts, _send_res_body) = send_res.into_parts();
    assert_eq!(send_res_parts.status, http::StatusCode::OK);
    assert_eq!(
        get_header_value(&send_res_parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&send_res_parts.headers, "access-control-allow-origin"),
        Some("*")
    );

    let (parts, body) = get_res_join_handle.await??.into_parts();
    let all_bytes: Vec<u8> = read_all_body(body).await;
    let expect = send_body_str.to_owned().into_bytes();
    assert_eq!(all_bytes, expect);

    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&parts.headers, "content-length"),
        Some(send_body_str.len().to_string().as_str())
    );
    assert_eq!(
        get_header_value(&parts.headers, "content-disposition"),
        None
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-origin"),
        Some("*")
    );
    assert_eq!(
        get_header_value(&parts.headers, "x-robots-tag"),
        Some("none")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-expose-headers"),
        None,
    );

    serve.shutdown_tx.send(()).expect("shutdown failed");
    Ok(())
}

// TODO: add tests when sender or receiver receive 400

#[it("should reject invalid n")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    {
        let send_body_str = "this is a content";
        let send_req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header("Content-Type", "text/plain")
            .uri(format!("http://{}/mypath?n=abc", serve.addr))
            .body(hyper::Body::from(send_body_str))?;

        let client = Client::new();
        let send_res = client.request(send_req).await?;
        let (send_res_parts, _send_res_body) = send_res.into_parts();
        assert_eq!(send_res_parts.status, http::StatusCode::BAD_REQUEST);
        assert_eq!(
            get_header_value(&send_res_parts.headers, "content-type"),
            Some("text/plain")
        );
        assert_eq!(
            get_header_value(&send_res_parts.headers, "access-control-allow-origin"),
            Some("*")
        );
    }

    {
        let get_req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(format!("http://{}/mypath?n=abc", serve.addr))
            .body(hyper::Body::empty())?;

        let client = Client::new();
        let get_res = client.request(get_req).await?;
        let (get_res_parts, _get_res_body) = get_res.into_parts();
        assert_eq!(get_res_parts.status, http::StatusCode::BAD_REQUEST);
        assert_eq!(
            get_header_value(&get_res_parts.headers, "content-type"),
            Some("text/plain")
        );
        assert_eq!(
            get_header_value(&get_res_parts.headers, "access-control-allow-origin"),
            Some("*")
        );
    }

    serve.shutdown_tx.send(()).expect("shutdown failed");
    Ok(())
}

#[it("should reject n > 1 because not supported yet")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    {
        let send_body_str = "this is a content";
        let send_req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header("Content-Type", "text/plain")
            .uri(format!("http://{}/mypath?n=2", serve.addr))
            .body(hyper::Body::from(send_body_str))?;

        let client = Client::new();
        let send_res = client.request(send_req).await?;
        let (send_res_parts, _send_res_body) = send_res.into_parts();
        assert_eq!(send_res_parts.status, http::StatusCode::BAD_REQUEST);
        assert_eq!(
            get_header_value(&send_res_parts.headers, "content-type"),
            Some("text/plain")
        );
        assert_eq!(
            get_header_value(&send_res_parts.headers, "access-control-allow-origin"),
            Some("*")
        );
    }

    {
        let get_req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(format!("http://{}/mypath?n=2", serve.addr))
            .body(hyper::Body::empty())?;

        let client = Client::new();
        let get_res = client.request(get_req).await?;
        let (get_res_parts, _get_res_body) = get_res.into_parts();
        assert_eq!(get_res_parts.status, http::StatusCode::BAD_REQUEST);
        assert_eq!(
            get_header_value(&get_res_parts.headers, "content-type"),
            Some("text/plain")
        );
        assert_eq!(
            get_header_value(&get_res_parts.headers, "access-control-allow-origin"),
            Some("*")
        );
    }

    serve.shutdown_tx.send(()).expect("shutdown failed");
    Ok(())
}

#[it("should pass X-Piping and attach Access-Control-Expose-Headers: X-Piping when sending with X-Piping")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/mypath", serve.addr).parse::<http::Uri>()?;

    let send_body_str = "this is a content";
    let send_req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header("Content-Type", "text/plain")
        .header("X-Piping", "mymetadata")
        .uri(uri.clone())
        .body(hyper::Body::from(send_body_str))?;

    let client = Client::new();
    let send_res = client.request(send_req).await?;
    let (send_res_parts, _send_res_body) = send_res.into_parts();
    assert_eq!(send_res_parts.status, http::StatusCode::OK);

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let (parts, body) = client.request(get_req).await?.into_parts();

    let all_bytes: Vec<u8> = read_all_body(body).await;

    let expect = send_body_str.to_owned().into_bytes();
    assert_eq!(all_bytes, expect);

    assert_eq!(
        get_header_value(&parts.headers, "content-type"),
        Some("text/plain")
    );
    assert_eq!(
        get_header_value(&parts.headers, "content-length"),
        Some(send_body_str.len().to_string().as_str())
    );
    assert_eq!(
        get_header_value(&parts.headers, "content-disposition"),
        None
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-allow-origin"),
        Some("*")
    );
    assert_eq!(
        get_header_value(&parts.headers, "x-robots-tag"),
        Some("none")
    );
    assert_eq!(
        get_header_value(&parts.headers, "access-control-expose-headers"),
        Some("X-Piping"),
    );
    assert_eq!(
        get_header_value(&parts.headers, "X-Piping"),
        Some("mymetadata"),
    );

    serve.shutdown_tx.send(()).expect("shutdown failed");
    Ok(())
}

#[it("should pass multiple X-Piping")]
async fn f() -> Result<(), BoxError> {
    let serve: Serve = serve().await;

    let uri = format!("http://{}/mypath", serve.addr).parse::<http::Uri>()?;

    let send_body_str = "this is a content";
    let send_req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .header("Content-Type", "text/plain")
        .header("X-Piping", "mymetadata1")
        .header("X-Piping", "mymetadata2")
        .header("X-Piping", "mymetadata3")
        .uri(uri.clone())
        .body(hyper::Body::from(send_body_str))?;

    let client = Client::new();
    let send_res = client.request(send_req).await?;
    let (send_res_parts, _send_res_body) = send_res.into_parts();
    assert_eq!(send_res_parts.status, http::StatusCode::OK);

    let get_req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(uri.clone())
        .body(hyper::Body::empty())?;
    let client = Client::new();
    let (parts, body) = client.request(get_req).await?.into_parts();

    let all_bytes: Vec<u8> = read_all_body(body).await;

    let expect = send_body_str.to_owned().into_bytes();
    assert_eq!(all_bytes, expect);

    assert_eq!(
        get_header_value(&parts.headers, "access-control-expose-headers"),
        Some("X-Piping"),
    );
    assert_eq!(
        parts
            .headers
            .get_all("X-Piping")
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["mymetadata1", "mymetadata2", "mymetadata3"],
    );

    serve.shutdown_tx.send(()).expect("shutdown failed");
    Ok(())
}
