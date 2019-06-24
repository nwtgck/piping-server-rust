extern crate hyper;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use hyper::{Body, Response, Server, Request, Method};
use hyper::rt::Future;
use hyper::service::{service_fn};
use futures::future;

struct SenderAndBody {
    // This is used to send body to the other. The other can be Piping Sever receiver if sender has first access. The other can be Piping Sever sender if receiver has first access.
    // The main reason to use this is wait for the other.
    response_sender: futures::sync::oneshot::Sender<Response<Body>>,
    // If Piping Server sender has first access, this object is filled, otherwise this is empty response.
    transferred_body: Response<Body>,
}

struct ConnectionFlags {
    has_sender: bool,
    has_receiver: bool,
}


// TODO: Use some logger instead of print!()s
fn main() {
    // TODO: Hard code
    let port = 8080;
    // TODO: Hard code
    let addr = ([0, 0, 0, 0], port).into();

    let path_to_sender_and_body: Arc<Mutex<HashMap<String, SenderAndBody>>> = Arc::new(Mutex::new(HashMap::new()));
    // TODO: Close detection and release disconnected senders or receivers and make it reusable
    let path_to_connection_flags: Arc<Mutex<HashMap<String, ConnectionFlags>>> = Arc::new(Mutex::new(HashMap::new()));

    let svc =  move || {
        let inner_path_to_sender_and_body = Arc::clone(&path_to_sender_and_body);
        let inner_path_to_connection_flags = Arc::clone(&path_to_connection_flags);

        let handler = move |req: Request<Body>| -> Box<dyn Future<Item=Response<Body>, Error=futures::Canceled> + Send> {
            let mut path_to_sender_and_body_guard = inner_path_to_sender_and_body.lock().unwrap();
            let mut path_to_connection_flags_guard = inner_path_to_connection_flags.lock().unwrap();

            println!("{} {}", req.method(), req.uri().path());
            match req.method() {
                &Method::GET => {
                    let path = req.uri().path();
                    let connection_flags = path_to_connection_flags_guard.get_mut(path);
                    let mut flags = match connection_flags {
                        Some(f) => {
                            f
                        },
                        None => {
                            let default_flags = ConnectionFlags {
                                has_sender: false,
                                has_receiver: false
                            };
                            path_to_connection_flags_guard.insert(path.to_string(), default_flags);
                            // NOTE: The unwrap() should be safe because after inserting
                            path_to_connection_flags_guard.get_mut(path).unwrap()
                        }
                    };

                    // If a receiver has been connected already
                    if flags.has_receiver {
                        Box::new(future::ok(Response::new(Body::from(format!("[ERROR] Another receiver has been connected on '{}'\n", path)))))
                    } else {
                        flags.has_receiver = true;
                        let sender_and_body_opt = path_to_sender_and_body_guard.remove(path);
                        match sender_and_body_opt {
                            Some(SenderAndBody{response_sender, transferred_body: body}) => {
                                response_sender.send(Response::new(Body::from("[INFO] Start sending\n"))).unwrap();
                                Box::new(future::ok(body))
                            },
                            None => {
                                // NOTE: c can be used as Future<Response<Body>>
                                let (p, c) = futures::oneshot::<Response<Body>>();
                                // NOTE: Body is dummy
                                path_to_sender_and_body_guard.insert(path.to_string(), SenderAndBody{
                                    response_sender: p,
                                    transferred_body: Response::new(Body::empty())
                                });
                                // This body will be filled by Piping Server sender by p
                                Box::new(c)
                            }
                        }
                    }
                },
                &Method::POST | &Method::PUT => {
                    let path = req.uri().path();
                    let connection_flags = path_to_connection_flags_guard.get_mut(path);
                    let mut flags = match connection_flags {
                        Some(f) => {
                            f
                        },
                        None => {
                            let default_flags = ConnectionFlags {
                                has_sender: false,
                                has_receiver: false
                            };
                            path_to_connection_flags_guard.insert(path.to_string(), default_flags);
                            // NOTE: The unwrap() should be safe because after inserting
                            path_to_connection_flags_guard.get_mut(path).unwrap()
                        }
                    };
                    // If a sender has been connected already
                    if flags.has_sender {
                        Box::new(future::ok(Response::new(Body::from(format!("[ERROR] Another sender has been connected on '{}'\n", path)))))
                    } else {
                        flags.has_sender = true;
                        let sender_and_body_opt = path_to_sender_and_body_guard.remove(path);
                        match sender_and_body_opt {
                            Some(SenderAndBody{response_sender, transferred_body: _}) => {
                                // Send request body into the existing receiver
                                response_sender.send(Response::new(req.into_body())).unwrap();
                                Box::new(future::ok(Response::new(Body::from("[INFO] Start sending\n"))))
                            },
                            None => {
                                // NOTE: c can be used as Future<Response<Body>>
                                let (p, c) = futures::oneshot::<Response<Body>>();
                                path_to_sender_and_body_guard.insert(path.to_string(), SenderAndBody{
                                    response_sender: p,
                                    // Transfer request body to future receiver
                                    transferred_body: Response::new(req.into_body())
                                });
                                // This body will be filled by Piping Server receiver by p
                                Box::new(c)
                            }
                        }
                    }
                },
                _ => {
                    println!("Unsupported method: {}", req.method());
                    Box::new(future::ok(Response::new(Body::from(format!("Unsupported method: {}\n", req.method())))))
                }
            }

        };
        service_fn(handler)
    };

    let server = Server::bind(&addr)
        .serve(svc)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("server is running on {}...", port);
    hyper::rt::run(server);
}
