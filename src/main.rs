use futures_util::stream::{StreamExt, TryStreamExt};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::convert::Infallible;
use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

mod piping_server;
mod req_res_handler;
mod util;
use piping_server::PipingServer;
use req_res_handler::req_res_handler;

/// Piping Server in Rust
#[derive(StructOpt, Debug)]
#[structopt(name = "piping-server")]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// HTTP port
    #[structopt(long, default_value = "8080")]
    http_port: u16,
    #[structopt(long)]
    enable_https: bool,
    /// HTTPS port
    #[structopt(long)]
    https_port: Option<u16>,
    #[structopt(long)]
    crt_path: Option<String>,
    #[structopt(long)]
    key_path: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse options
    let opt = Opt::from_args();

    let mut tcp: TcpListener;
    let tls_acceptor: TlsAcceptor;

    let piping_server = &PipingServer::new();

    env_logger::init();

    let https_server = if opt.enable_https {
        if let (Some(https_port), Some(crt_path), Some(key_path)) =
            (opt.https_port, opt.crt_path, opt.key_path)
        {
            let tls_cfg = util::load_tls_config(crt_path, key_path)?;

            let addr: std::net::SocketAddr = ([0, 0, 0, 0], https_port).into();
            // Create a TCP listener via tokio.
            tcp = TcpListener::bind(&addr).await?;
            tls_acceptor = TlsAcceptor::from(std::sync::Arc::new(tls_cfg));
            // let tls_acceptor = tls_acceptor_opt.as_ref().unwrap();
            // Prepare a long-running future stream to accept and serve clients.
            let incoming_tls_stream = tcp
                .incoming()
                .map_err(|e| util::make_io_error(format!("Incoming failed: {:?}", e)))
                // (base: https://github.com/cloudflare/wrangler/pull/1485/files)
                .filter_map(|s| async {
                    let client = match s {
                        Ok(x) => x,
                        Err(e) => {
                            log::error!("Failed to accept client: {}", e);
                            return None;
                        }
                    };
                    match tls_acceptor.accept(client).await {
                        Ok(x) => Some(Ok::<_, std::io::Error>(x)),
                        Err(e) => {
                            log::error!("Client connection error: {}", e);
                            None
                        }
                    }
                });
            let https_svc = make_service_fn(move |_| {
                let piping_server = piping_server.clone();
                async move {
                    let handler = req_res_handler(move |req, res_sender| {
                        piping_server.clone().handler(req, res_sender)
                    });
                    Ok::<_, Infallible>(service_fn(handler))
                }
            });
            let https_server = Server::builder(util::HyperAcceptor {
                acceptor: Box::pin(incoming_tls_stream),
            })
            .serve(https_svc);
            futures::future::Either::Left(https_server)
        } else {
            return Err(util::make_io_error(
                "--https-port, --crt-path and --key-path should be specified".to_owned(),
            ));
        }
    } else {
        futures::future::Either::Right(futures::future::ok(()))
    };

    let http_svc = make_service_fn(move |_| {
        let piping_server = piping_server.clone();
        async move {
            let handler = req_res_handler(move |req, res_sender| {
                piping_server.clone().handler(req, res_sender)
            });
            Ok::<_, Infallible>(service_fn(handler))
        }
    });
    let http_server = Server::bind(&([0, 0, 0, 0], opt.http_port).into()).serve(http_svc);

    log::info!("HTTP server is running on {}...", opt.http_port);
    if let Some(https_port) = opt.https_port {
        log::info!("HTTPS server is running on {:?}...", https_port);
    }
    match futures::future::join(http_server, https_server).await {
        (Err(e), _) => return Err(util::make_io_error(format!("HTTP server error: {}", e))),
        (_, Err(e)) => return Err(util::make_io_error(format!("HTTPS server error: {}", e))),
        _ => (),
    }
    Ok(())
}
