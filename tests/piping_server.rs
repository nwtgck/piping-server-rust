use futures::channel::oneshot;
use hyper::service::{make_service_fn, service_fn};
use hyper::Client;
use hyper::Server;
use specit::tokio_it as it;
use std::convert::Infallible;

use piping_server::piping_server::PipingServer;
use piping_server::req_res_handler::req_res_handler;
use std::net::SocketAddr;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

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
            let handler =
                req_res_handler(move |req, res_sender| piping_server.handler(req, res_sender));
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

    // Body should contains "Piping"
    let body_string = String::from_utf8(read_all_body(body).await)?;
    assert!(body_string.contains("Piping"));

    // Content-Type is "text/html"
    let content_type: &str = parts.headers.get("content-type").unwrap().to_str()?;
    assert_eq!(content_type, "text/html");

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

    let content_type: &str = parts.headers.get("content-type").unwrap().to_str()?;
    assert_eq!(content_type, "text/plain");
    let access_control_allow_origin: &str = parts
        .headers
        .get("access-control-allow-origin")
        .unwrap()
        .to_str()?;
    assert_eq!(access_control_allow_origin, "*");

    serve.shutdown().await?;
    Ok(())
}
