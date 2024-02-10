use clap::Parser as _;
use std::net::SocketAddr;
use std::sync::Arc;

use piping_server::piping_server::PipingServer;
use piping_server::util;

/// Piping Server in Rust
#[derive(clap::Parser, Debug, Clone)]
#[clap(name = "piping-server")]
#[clap(about, version)]
struct Args {
    /// Bind address, either IPv4 or IPv6 (e.g. 127.0.0.1, ::1)
    #[clap(long, default_value = "0.0.0.0")]
    host: std::net::IpAddr,
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
async fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Args::parse();

    // Set default log level
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let piping_server = PipingServer::new();

    let version = env!("CARGO_PKG_VERSION");
    log::info!("Piping Server (Rust) {version}");

    let serve_http = {
        let piping_server = piping_server.clone();
        let args = args.clone();
        util::future_with_output_type::<anyhow::Result<()>, _>(async move {
            let tcp_listener =
                tokio::net::TcpListener::bind(SocketAddr::new(args.host, args.http_port)).await?;
            log::info!("HTTP server is listening on {}...", args.http_port);
            let piping_server_service =
                hyper::service::service_fn(move |req| piping_server.clone().handle(false, req));

            loop {
                let (stream, _) = tcp_listener.accept().await?;
                let piping_server_service = piping_server_service.clone();
                tokio::task::spawn(async move {
                    if let Err(err) = hyper_util::server::conn::auto::Builder::new(
                        hyper_util::rt::tokio::TokioExecutor::new(),
                    )
                    .serve_connection(hyper_util::rt::TokioIo::new(stream), piping_server_service)
                    .await
                    {
                        log::error!("Failed to serve HTTP connection: {err:?}");
                    }
                });
            }
        })
    };

    let serve_https = async move {
        if args.enable_https {
            if let (Some(https_port), Some(crt_path), Some(key_path)) =
                (args.https_port, args.crt_path, args.key_path)
            {
                let piping_server = piping_server.clone();

                let tokio_handle = tokio::runtime::Handle::current();
                let tls_cfg_rwlock_arc: Arc<tokio::sync::RwLock<Arc<rustls::ServerConfig>>> =
                    util::hot_reload_tls_cfg(tokio_handle, crt_path, key_path);

                let tcp_listener =
                    tokio::net::TcpListener::bind(SocketAddr::new(args.host, https_port)).await?;
                log::info!("HTTPS server is listening on {https_port}...");

                let piping_server_service =
                    hyper::service::service_fn(move |req| piping_server.clone().handle(true, req));

                loop {
                    let (stream, _) = tcp_listener.accept().await?;
                    let rustls_config = tls_cfg_rwlock_arc.clone().read().await.clone();
                    let stream = match tokio_rustls::TlsAcceptor::from(rustls_config)
                        .accept(stream)
                        .await
                    {
                        Ok(stream) => stream,
                        Err(err) => {
                            log::error!("Failed to accept TLS connection: {err:?}");
                            continue;
                        }
                    };
                    let piping_server_service = piping_server_service.clone();
                    tokio::task::spawn(async move {
                        if let Err(err) = hyper_util::server::conn::auto::Builder::new(
                            hyper_util::rt::tokio::TokioExecutor::new(),
                        )
                        .serve_connection(
                            hyper_util::rt::TokioIo::new(stream),
                            piping_server_service,
                        )
                        .await
                        {
                            log::error!("Failed to serve HTTPS connection: {err:?}");
                        }
                    });
                }
            } else {
                anyhow::bail!("--https-port, --crt-path and --key-path should be specified");
            }
        } else {
            Ok(())
        }
    };

    let _: ((), ()) = futures::try_join!(serve_http, serve_https)?;
    Ok(())
}
