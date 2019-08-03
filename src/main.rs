use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use hyper::{Body, Response, Server, Request, Method, Chunk};
use hyper::rt::Future;
use hyper::service::{service_fn};
use futures::sync::oneshot;
use structopt::StructOpt;
use futures::Async;

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
    res_sender:  oneshot::Sender<Response<Body>>,
}


// NOTE: oneshot::Receiver can be Future
fn req_res_handler<F>(mut handler: F) -> impl FnMut(Request<Body>) -> oneshot::Receiver<Response<Body>> where
    F: FnMut(Request<Body>, oneshot::Sender<Response<Body>>) -> ()
{
    move |req| {
        let (res_sender, res_receiver) = oneshot::channel::<Response<Body>>();
        handler(req, res_sender);
        res_receiver
    }
}

struct FinishDetectableBody {
    body: Body,
    finish_notifier: Option<oneshot::Sender<()>>,
}

impl futures::stream::Stream for FinishDetectableBody {
    type Item = Chunk;
    type Error = hyper::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.body.poll() {
            // If body is finished
            Ok(Async::Ready(None)) => {
                // Notify finish
                if let Some(notifier) = self.finish_notifier.take() {
                    notifier.send(()).unwrap();
                }
                Ok(Async::Ready(None))
            },
            r@ _ => r
        }
    }
}

impl FinishDetectableBody {
    fn new(body: Body, finish_notifier: oneshot::Sender<()>) -> FinishDetectableBody {
        FinishDetectableBody {
            body,
            finish_notifier: Some(finish_notifier)
        }
    }
}

fn transfer(path: String, sender_req_res: ReqRes, receiver_req_res: ReqRes) {
    println!("Transfer start: '{}'", path);

    // For streaming sender's response body
    let (mut sender_res_body_sender, sender_res_body) = Body::channel();
    // For notifying and waiting for sender's request body
    let (sender_req_body_finish_notifier, sender_req_body_finish_waiter) = oneshot::channel::<()>();

    // Notify sender when sending starts
    sender_res_body_sender.send_data(Chunk::from("[INFO] Start sending...\n")).unwrap();
    // Create receiver's body
    let receiver_res_body = Body::wrap_stream(FinishDetectableBody::new(
        sender_req_res.req.into_body(),
        sender_req_body_finish_notifier
    ));
    // Create receiver's response
    let receiver_res = Response::builder()
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

    // Wait for sender's request body finished
    hyper::rt::spawn(sender_req_body_finish_waiter.then(move |_| {
        // Notify sender when sending finished
        sender_res_body_sender.send_data(Chunk::from("[INFO] Sent successfully!\n")).unwrap();
        println!("Transfer end: '{}'", path);
        Ok(())
    }));
}

// TODO: Use some logger instead of print!()s
fn main() {
    // Parse options
    let opt = Opt::from_args();

    let port = opt.http_port;
    let addr = ([0, 0, 0, 0], port).into();

    let path_to_sender: Arc<Mutex<HashMap<String, ReqRes>>> = Arc::new(Mutex::new(HashMap::new()));
    let path_to_receiver: Arc<Mutex<HashMap<String, ReqRes>>> = Arc::new(Mutex::new(HashMap::new()));


    let svc = move || {
        let path_to_sender = Arc::clone(&path_to_sender);
        let path_to_receiver = Arc::clone(&path_to_receiver);

        let handler = req_res_handler(move |req, res_sender| {
            let mut path_to_sender_guard = path_to_sender.lock().unwrap();
            let mut path_to_receiver_guard = path_to_receiver.lock().unwrap();

            let path = req.uri().path();

            println!("{} {}", req.method(), req.uri().path());
            match req.method() {
                &Method::GET => {
                    // If a receiver has been connected already
                    if path_to_receiver_guard.contains_key(path) {
                        let res = Response::builder()
                            .status(400)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(Body::from(format!("[ERROR] Another receiver has been connected on '{}'.\n", path)))
                            .unwrap();
                        res_sender.send(res).unwrap();
                        return;
                    }
                    match path_to_sender_guard.remove(path) {
                        // If sender is found
                        Some(sender_req_res) => {
                            transfer(path.to_string(), sender_req_res, ReqRes{req, res_sender});
                        },
                        // If sender is not found
                        None => {
                            path_to_receiver_guard.insert(path.to_string(), ReqRes {
                                req,
                                res_sender,
                            });
                        }
                    }
                },
                &Method::POST | &Method::PUT => {
                    // If a sender has been connected already
                    if path_to_sender_guard.contains_key(path) {
                        let res = Response::builder()
                            .status(400)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(Body::from(format!("[ERROR] Another sender has been connected on '{}'.\n", path)))
                            .unwrap();
                        res_sender.send(res).unwrap();
                        return;
                    }
                    match path_to_receiver_guard.remove(path) {
                        // If receiver is found
                        Some(receiver_req_res) => {
                            transfer(path.to_string(),ReqRes{req, res_sender}, receiver_req_res);
                        },
                        // If receiver is not found
                        None => {
                            path_to_sender_guard.insert(path.to_string(), ReqRes{req, res_sender});
                        }
                    }
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
        });
        service_fn(handler)
    };

    let server = Server::bind(&addr)
        .serve(svc)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("server is running on {}...", port);
    hyper::rt::run(server);
}
