use clap::Parser;
use core::convert::Infallible;
use futures::stream::{StreamExt, TryStreamExt};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use piping_server::piping_server::PipingServer;
use piping_server::req_res_handler::req_res_handler;
use piping_server::util;

/// Piping Server in Rust
#[derive(clap::Parser, Debug)]
#[clap(name = "piping-server")]
#[clap(about, version)]
#[clap(global_setting(clap::AppSettings::DeriveDisplayOrder))]
struct Args {
    /// HTTP port
    #[clap(long, default_value = "8080")]
    http_port: u16,
    #[clap(long)]
    /// Enable HTTPS
    enable_https: bool,
    /// HTTPS port
    #[clap(long)]
    https_port: Option<u16>,
    /// Certification path
    #[clap(long)]
    crt_path: Option<String>,
    /// Private key path
    #[clap(long)]
    key_path: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse arguments
    let args = Args::parse();

    let mut tcp: TcpListener;
    let tls_cfg_rwlock_arc: Arc<RwLock<Arc<rustls::ServerConfig>>>;

    let piping_server = &PipingServer::new();

    // Set default log level
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!(
        "Piping Server (Rust) {version}",
        version = env!("CARGO_PKG_VERSION")
    );

    let https_server = if args.enable_https {
        if let (Some(https_port), Some(crt_path), Some(key_path)) =
            (args.https_port, args.crt_path, args.key_path)
        {
            tls_cfg_rwlock_arc = util::hot_reload_tls_cfg(crt_path, key_path);

            let addr: std::net::SocketAddr = ([0, 0, 0, 0], https_port).into();
            // Create a TCP listener via tokio.
            tcp = TcpListener::bind(&addr).await?;
            // Prepare a long-running future stream to accept and serve clients.
            let incoming_tls_stream = util::TokioIncoming::new(&mut tcp)
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

                    let tls_cfg: Arc<rustls::ServerConfig> =
                        (*tls_cfg_rwlock_arc.read().unwrap()).clone();
                    match TlsAcceptor::from(tls_cfg).accept(client).await {
                        Ok(x) => Some(Ok::<_, std::io::Error>(x)),
                        Err(e) => {
                            log::error!("Client connection error: {}", e);
                            None
                        }
                    }
                });
            let https_svc = make_service_fn(move |_| {
                let piping_server = piping_server.clone();
                let handler = req_res_handler(move |req, res_sender| {
                    piping_server.handler(true, req, res_sender)
                });
                futures::future::ok::<_, Infallible>(service_fn(handler))
            });
            let https_server = Server::builder(util::HyperAcceptor {
                acceptor: incoming_tls_stream,
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

    let http_svc = make_service_fn(|_| {
        let piping_server = piping_server.clone();
        let handler =
            req_res_handler(move |req, res_sender| piping_server.handler(false, req, res_sender));
        futures::future::ok::<_, Infallible>(service_fn(handler))
    });
    let http_server = Server::bind(&([0, 0, 0, 0], args.http_port).into()).serve(http_svc);

    log::info!("HTTP server is running on {}...", args.http_port);
    if let Some(https_port) = args.https_port {
        log::info!("HTTPS server is running on {:?}...", https_port);
    }
    match futures::future::join(http_server, https_server).await {
        (Err(e), _) => return Err(util::make_io_error(format!("HTTP server error: {}", e))),
        (_, Err(e)) => return Err(util::make_io_error(format!("HTTPS server error: {}", e))),
        _ => (),
    }
    Ok(())
}
