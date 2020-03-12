#![feature(fn_traits)]
#![feature(unboxed_closures)]

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::future::Future;

use hyper::{Body, Response, Server, Request, Method};
use hyper::body::Bytes;
use hyper::service::{service_fn, make_service_fn};
use futures::channel::oneshot;
use structopt::StructOpt;

mod util;
use util::{OptionHeaderBuilder, FinishDetectableBody};
use std::convert::Infallible;
use futures::FutureExt;

/// Piping Server in Rust
#[derive(StructOpt, Debug)]
#[structopt(name = "piping-server")]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// HTTP port
    #[structopt(long, default_value = "8080")]
    http_port: u16,
}


struct ReqRes {
    req: Request<Body>,
    res_sender: oneshot::Sender<Response<Body>>,
}

struct HoldOneReturnThatClosure<T> {
    value: T,
}

impl<T> std::ops::FnOnce<((),)> for HoldOneReturnThatClosure<T> {
    type Output = T;
    extern "rust-call" fn call_once(self, _args: ((),)) -> Self::Output {
        self.value
    }
}

// NOTE: futures::future::Then<..., oneshot::Receiver, ...> can be a Future
fn req_res_handler<F, Fut>(mut handler: F) -> impl (FnMut(Request<Body>) -> futures::future::Then< Fut, oneshot::Receiver<Response<Body>>, HoldOneReturnThatClosure<oneshot::Receiver<Response<Body>>> > ) where
    F: FnMut(Request<Body>, oneshot::Sender<Response<Body>>) -> Fut,
    Fut: Future<Output=()>
{
    move |req| {
        let (res_sender, res_receiver) = oneshot::channel::<Response<Body>>();
        handler(req, res_sender).then(HoldOneReturnThatClosure{ value: res_receiver })
    }
}


async fn transfer(path: String, sender_req_res: ReqRes, receiver_req_res: ReqRes) {
    println!("Transfer start: '{}'", path);

    // For streaming sender's response body
    let (mut sender_res_body_sender, sender_res_body) = Body::channel();
    // For notifying and waiting for sender's request body
    let (sender_req_body_finish_notifier, sender_req_body_finish_waiter) = oneshot::channel::<()>();

    // Get sender's header
    let sender_header = sender_req_res.req.headers();
    // Get sender's header values
    let sender_content_type = sender_header.get("content-type").cloned();
    let sender_content_length = sender_header.get("content-length").cloned();
    let sender_content_disposition = sender_header.get("content-disposition").cloned();

    // Notify sender when sending starts
    sender_res_body_sender.send_data(Bytes::from("[INFO] Start sending...\n")).await;
    // Create receiver's body
    let receiver_res_body = Body::wrap_stream::<FinishDetectableBody, Bytes, http::Error>(FinishDetectableBody::new(
        sender_req_res.req.into_body(),
        sender_req_body_finish_notifier
    ));

    // Create receiver's response
    let receiver_res = Response::builder()
        .option_header("Content-Type", sender_content_type)
        .option_header("Content-Length", sender_content_length)
        .option_header("Content-Disposition", sender_content_disposition)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Expose-Headers", "Content-Length, Content-Type")
        .body(receiver_res_body)
        .unwrap();
    // Return response to receiver
    receiver_req_res.res_sender.send(receiver_res).unwrap();

    // Create sender's response
    let sender_res = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .body(sender_res_body)
        .unwrap();
    // Return response to sender
    sender_req_res.res_sender.send(sender_res).unwrap();

    tokio::task::spawn(async move {
        // Wait for sender's request body finished
        sender_req_body_finish_waiter.await;
        // Notify sender when sending finished
        sender_res_body_sender.send_data(Bytes::from("[INFO] Sent successfully!\n")).await;
        println!("Transfer end: '{}'", path);
    });
}

// TODO: Use some logger instead of print!()s
#[tokio::main]
async fn main() {
    // Parse options
    let opt = Opt::from_args();

    let port = opt.http_port;
    let addr: std::net::SocketAddr = ([0, 0, 0, 0], port).into();

    let path_to_sender: Arc<Mutex<HashMap<String, ReqRes>>> = Arc::new(Mutex::new(HashMap::new()));
    let path_to_receiver: Arc<Mutex<HashMap<String, ReqRes>>> = Arc::new(Mutex::new(HashMap::new()));

    let svc = make_service_fn( move |_| {
        let path_to_sender = Arc::clone(&path_to_sender);
        let path_to_receiver = Arc::clone(&path_to_receiver);
        async move {
            let handler = req_res_handler( move |req, res_sender|
                {
                    let path_to_sender = Arc::clone(&path_to_sender);
                    let path_to_receiver = Arc::clone(&path_to_receiver);
                    async move {
                        let path = req.uri().path();

                        println!("{} {}", req.method(), req.uri().path());
                        match req.method() {
                            &Method::GET => {
                                match path {
                                    "/" => {
                                        let res = Response::builder()
                                            .status(200)
                                            .header("Content-Type", "text/html")
                                            .header("Access-Control-Allow-Origin", "*")
                                            .body(Body::from(include_str!("../resource/index.html")))
                                            .unwrap();
                                        res_sender.send(res).unwrap();
                                    },
                                    "/version" => {
                                        let version: &'static str = env!("CARGO_PKG_VERSION");
                                        let res = Response::builder()
                                            .status(200)
                                            .header("Content-Type", "text/plain")
                                            .header("Access-Control-Allow-Origin", "*")
                                            .body(Body::from(format!("{} in Rust (Hyper)", version)))
                                            .unwrap();
                                        res_sender.send(res).unwrap();
                                    },
                                    _ => {
                                        let receiver_connected: bool = {
                                            let mut path_to_receiver_guard = path_to_receiver.lock().unwrap();
                                            path_to_receiver_guard.contains_key(path)
                                        };
                                        // If a receiver has been connected already
                                        if receiver_connected {
                                            let res = Response::builder()
                                                .status(400)
                                                .header("Access-Control-Allow-Origin", "*")
                                                .body(Body::from(format!("[ERROR] Another receiver has been connected on '{}'.\n", path)))
                                                .unwrap();
                                            res_sender.send(res).unwrap();
                                            return;
                                        }
                                        let sender = {
                                            let mut path_to_sender_guard = path_to_sender.lock().unwrap();
                                            path_to_sender_guard.remove(path)
                                        };
                                        match sender {
                                            // If sender is found
                                            Some(sender_req_res) => {
                                                transfer(path.to_string(), sender_req_res, ReqRes{req, res_sender}).await;
                                            },
                                            // If sender is not found
                                            None => {
                                                let mut path_to_receiver_guard = path_to_receiver.lock().unwrap();
                                                path_to_receiver_guard.insert(path.to_string(), ReqRes {
                                                    req,
                                                    res_sender,
                                                });
                                            }
                                        }
                                    }
                                }

                            },
                            &Method::POST | &Method::PUT => {
                                let sender_connected: bool = {
                                    let mut path_to_sender_guard = path_to_sender.lock().unwrap();
                                    path_to_sender_guard.contains_key(path)
                                };
                                // If a sender has been connected already
                                if sender_connected {
                                    let res = Response::builder()
                                        .status(400)
                                        .header("Access-Control-Allow-Origin", "*")
                                        .body(Body::from(format!("[ERROR] Another sender has been connected on '{}'.\n", path)))
                                        .unwrap();
                                    res_sender.send(res).unwrap();
                                    return;
                                }
                                let receiver = {
                                    let mut path_to_receiver_guard = path_to_receiver.lock().unwrap();
                                    path_to_receiver_guard.remove(path)
                                };
                                match receiver {
                                    // If receiver is found
                                    Some(receiver_req_res) => {
                                        transfer(path.to_string(),ReqRes{req, res_sender}, receiver_req_res).await;
                                    },
                                    // If receiver is not found
                                    None => {
                                        let mut path_to_sender_guard = path_to_sender.lock().unwrap();
                                        path_to_sender_guard.insert(path.to_string(), ReqRes{req, res_sender});
                                    }
                                }
                            },
                            &Method::OPTIONS => {
                                // Response for Preflight request
                                let res = Response::builder()
                                    .status(200)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .header("Access-Control-Allow-Methods", "GET, HEAD, POST, PUT, OPTIONS")
                                    .header("Access-Control-Allow-Headers", "Content-Type, Content-Disposition")
                                    .header("Access-Control-Max-Age", 86400)
                                    .header("Content-Length", 0)
                                    .body(Body::empty())
                                    .unwrap();
                                res_sender.send(res).unwrap();
                            },
                            _ => {
                                println!("Unsupported method: {}", req.method());
                                let res = Response::builder()
                                    .status(400)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .body(Body::from(format!("[ERROR] Unsupported method: {}.\n", req.method())))
                                    .unwrap();
                                res_sender.send(res).unwrap();
                            }
                        }
                    }
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
