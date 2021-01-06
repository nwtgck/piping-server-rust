use core::pin::Pin;
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::future::FutureExt;
use futures::stream::{Stream, StreamExt};
use http::{Method, Request, Response};
use hyper::body::Bytes;
use hyper::Body;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::util::{
    finish_detectable_stream, one_stream, FinishDetectableStream, OptionHeaderBuilder,
};

mod reserved_paths {
    crate::with_values! {
        pub const INDEX: &'static str = "/";
        pub const VERSION: &'static str = "/version";
        pub const FAVICON_ICO: &'static str = "/favicon.ico";
        pub const ROBOTS_TXT: &'static str = "/robots.txt";
    }
}

struct DataSender {
    req: Request<Body>,
    res_body_streams_sender: RwLock<
        mpsc::UnboundedSender<Pin<Box<dyn Stream<Item = Result<Bytes, std::convert::Infallible>> + Send>>>,
    >,
}

struct DataReceiver {
    res_sender: oneshot::Sender<Response<Body>>,
}

pub struct PipingServer {
    path_to_sender: Arc<RwLock<HashMap<String, DataSender>>>,
    path_to_receiver: Arc<RwLock<HashMap<String, DataReceiver>>>,
}

impl PipingServer {
    pub fn new() -> Self {
        PipingServer {
            path_to_sender: Arc::new(RwLock::new(HashMap::new())),
            path_to_receiver: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn clone(&self) -> Self {
        PipingServer {
            path_to_sender: Arc::clone(&self.path_to_sender),
            path_to_receiver: Arc::clone(&self.path_to_receiver),
        }
    }

    pub fn handler(
        &self,
        req: Request<Body>,
        res_sender: oneshot::Sender<Response<Body>>,
    ) -> impl std::future::Future<Output = ()> {
        let path_to_sender = Arc::clone(&self.path_to_sender);
        let path_to_receiver = Arc::clone(&self.path_to_receiver);
        async move {
            let path = req.uri().path();

            log::info!("{} {}", req.method(), req.uri().path());
            match req.method() {
                &Method::GET => {
                    match path {
                        reserved_paths::INDEX => {
                            let res = Response::builder()
                                .status(200)
                                .header("Content-Type", "text/html")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(Body::from(include_str!("../resource/index.html")))
                                .unwrap();
                            res_sender.send(res).unwrap();
                        }
                        reserved_paths::VERSION => {
                            let version: &'static str = env!("CARGO_PKG_VERSION");
                            let res = Response::builder()
                                .status(200)
                                .header("Content-Type", "text/plain")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(Body::from(format!("{} in Rust (Hyper)\n", version)))
                                .unwrap();
                            res_sender.send(res).unwrap();
                        }
                        reserved_paths::FAVICON_ICO => {
                            let res = Response::builder().status(204).body(Body::empty()).unwrap();
                            res_sender.send(res).unwrap();
                        }
                        reserved_paths::ROBOTS_TXT => {
                            let res = Response::builder().status(404).body(Body::empty()).unwrap();
                            res_sender.send(res).unwrap();
                        }
                        _ => {
                            if let Some(value) = req.headers().get("service-worker") {
                                if value == http::HeaderValue::from_static("script") {
                                    // Reject Service Worker registration
                                    let res = Response::builder()
                                        .status(400)
                                        .header("Access-Control-Allow-Origin", "*")
                                        .body(Body::from(
                                            "[ERROR] Service Worker registration is rejected.\n",
                                        ))
                                        .unwrap();
                                    res_sender.send(res).unwrap();
                                    return;
                                }
                            }
                            let receiver_connected: bool =
                                path_to_receiver.read().unwrap().contains_key(path);
                            // If a receiver has been connected already
                            if receiver_connected {
                                let res = Response::builder()
                                    .status(400)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .body(Body::from(format!(
                                        "[ERROR] Another receiver has been connected on '{}'.\n",
                                        path
                                    )))
                                    .unwrap();
                                res_sender.send(res).unwrap();
                                return;
                            }
                            let sender = path_to_sender.write().unwrap().remove(path);
                            match sender {
                                // If sender is found
                                Some(data_sender) => {
                                    data_sender
                                        .res_body_streams_sender
                                        .write()
                                        .unwrap()
                                        .unbounded_send(
                                            one_stream(Ok(Bytes::from(
                                                "[INFO] A receiver was connected.\n",
                                            )))
                                            .boxed(),
                                        )
                                        .unwrap();
                                    transfer(
                                        path.to_string(),
                                        data_sender,
                                        DataReceiver { res_sender },
                                    );
                                }
                                // If sender is not found
                                None => {
                                    path_to_receiver
                                        .write()
                                        .unwrap()
                                        .insert(path.to_string(), DataReceiver { res_sender });
                                }
                            }
                        }
                    }
                }
                &Method::POST | &Method::PUT => {
                    if reserved_paths::VALUES.contains(&path) {
                        // Reject reserved path sending
                        let res = Response::builder()
                            .status(400)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(Body::from(format!(
                                "[ERROR] Cannot send to the reserved path '{}'. (e.g. '/mypath123')\n",
                                path
                            )))
                            .unwrap();
                        res_sender.send(res).unwrap();
                        return;
                    }
                    let sender_connected: bool = path_to_sender.read().unwrap().contains_key(path);
                    // If a sender has been connected already
                    if sender_connected {
                        let res = Response::builder()
                            .status(400)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(Body::from(format!(
                                "[ERROR] Another sender has been connected on '{}'.\n",
                                path
                            )))
                            .unwrap();
                        res_sender.send(res).unwrap();
                        return;
                    }

                    let (tx, rx) = mpsc::unbounded::<
                        Pin<Box<dyn Stream<Item = Result<Bytes, std::convert::Infallible>> + Send>>,
                    >();
                    let body = hyper::body::Body::wrap_stream(rx.flatten());
                    let sender_res = Response::builder()
                        .header("Access-Control-Allow-Origin", "*")
                        .body(body)
                        .unwrap();
                    res_sender.send(sender_res).unwrap();

                    let receiver = path_to_receiver.write().unwrap().remove(path);
                    match receiver {
                        // If receiver is found
                        Some(data_receiver) => {
                            tx.unbounded_send(
                                one_stream(Ok(Bytes::from(
                                    "[INFO] 1 receiver(s) has/have been connected.\n",
                                )))
                                .boxed(),
                            )
                            .unwrap();
                            transfer(
                                path.to_string(),
                                DataSender {
                                    req,
                                    res_body_streams_sender: RwLock::new(tx),
                                },
                                data_receiver,
                            );
                        }
                        // If receiver is not found
                        None => {
                            tx.unbounded_send(
                                one_stream(Ok(Bytes::from(
                                    "[INFO] Waiting for 1 receiver(s)...\n",
                                )))
                                .boxed(),
                            )
                            .unwrap();
                            path_to_sender.write().unwrap().insert(
                                path.to_string(),
                                DataSender {
                                    req,
                                    res_body_streams_sender: RwLock::new(tx),
                                },
                            );
                        }
                    }
                }
                &Method::OPTIONS => {
                    // Response for Preflight request
                    let res = Response::builder()
                        .status(200)
                        .header("Access-Control-Allow-Origin", "*")
                        .header(
                            "Access-Control-Allow-Methods",
                            "GET, HEAD, POST, PUT, OPTIONS",
                        )
                        .header(
                            "Access-Control-Allow-Headers",
                            "Content-Type, Content-Disposition",
                        )
                        .header("Access-Control-Max-Age", 86400)
                        .header("Content-Length", 0)
                        .body(Body::empty())
                        .unwrap();
                    res_sender.send(res).unwrap();
                }
                _ => {
                    log::info!("Unsupported method: {}", req.method());
                    let res = Response::builder()
                        .status(400)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Body::from(format!(
                            "[ERROR] Unsupported method: {}.\n",
                            req.method()
                        )))
                        .unwrap();
                    res_sender.send(res).unwrap();
                }
            }
        }
    }
}

fn transfer(path: String, data_sender: DataSender, data_receiver: DataReceiver) {
    log::info!("Transfer start: '{}'", path);

    let data_sender_req = data_sender.req;

    // Get sender's header
    let sender_header = data_sender_req.headers();
    // Get sender's header values
    let sender_content_type = sender_header.get("content-type").cloned();
    let sender_content_length = sender_header.get("content-length").cloned();
    let sender_content_disposition = sender_header.get("content-disposition").cloned();

    // The finish_waiter will tell when the body is finished
    let (finish_detectable_body, sender_req_body_finish_waiter) =
        finish_detectable_stream(data_sender_req.into_body());

    // Create receiver's body
    let receiver_res_body = Body::wrap_stream::<FinishDetectableStream<Body>, Bytes, hyper::Error>(
        finish_detectable_body,
    );

    // Create receiver's response
    let receiver_res = Response::builder()
        .option_header("Content-Type", sender_content_type)
        .option_header("Content-Length", sender_content_length)
        .option_header("Content-Disposition", sender_content_disposition)
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Expose-Headers",
            "Content-Length, Content-Type",
        )
        .body(receiver_res_body)
        .unwrap();
    // Return response to receiver
    data_receiver.res_sender.send(receiver_res).unwrap();

    data_sender
        .res_body_streams_sender
        .write()
        .unwrap()
        .unbounded_send(
            one_stream(Ok(Bytes::from(
                "[INFO] Start sending to 1 receiver(s)...\n",
            )))
            .chain(
                // Wait for sender's request body finished
                sender_req_body_finish_waiter
                    .into_stream()
                    .map(|_| Ok(Bytes::new())),
            )
            .chain(
                // Notify sender when sending finished
                one_stream(Ok(Bytes::from("[INFO] Sent successfully!\n"))),
            )
            .chain(one_stream(Ok(Bytes::new())).map(move |x| {
                log::info!("Transfer end: '{}'", path);
                x
            }))
            .boxed(),
        )
        .unwrap();
}
