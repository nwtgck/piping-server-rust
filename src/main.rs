use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::convert::Infallible;
use structopt::StructOpt;

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
}

// TODO: Use some logger instead of print!()s
#[tokio::main]
async fn main() {
    // Parse options
    let opt = Opt::from_args();

    let port = opt.http_port;
    let addr: std::net::SocketAddr = ([0, 0, 0, 0], port).into();

    let piping_server = PipingServer::new();

    let svc = make_service_fn(move |_| {
        let piping_server = piping_server.clone();
        async move {
            let handler = req_res_handler(move |req, res_sender| {
                piping_server.clone().handler(req, res_sender)
            });
            Ok::<_, Infallible>(service_fn(handler))
        }
    });
    let server = Server::bind(&addr)
        .serve(svc);

    println!("server is running on {}...", port);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
