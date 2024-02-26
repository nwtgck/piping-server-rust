use anyhow::anyhow;
use futures::{SinkExt as _, StreamExt as _, TryStreamExt as _};
use http_body_util::BodyExt as _;
use hyper::body::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

use crate::dynamic_resources;
use crate::util::{
    empty_body, finish_detectable_body, full_body, query_param_to_hash_map, FinishDetectableBody,
    HeaderValuesBuilder, OptionHeaderBuilder,
};

pub mod reserved_paths {
    crate::with_values! {
        pub const INDEX: &'static str = "/";
        pub const NO_SCRIPT: &'static str = "/noscript";
        pub const VERSION: &'static str = "/version";
        pub const HELP: &'static str = "/help";
        pub const FAVICON_ICO: &'static str = "/favicon.ico";
        pub const ROBOTS_TXT: &'static str = "/robots.txt";
    }
}

pub const NO_SCRIPT_PATH_QUERY_PARAMETER_NAME: &str = "path";

struct DataSender {
    // request
    req_headers: http::header::HeaderMap,
    req_body: hyper::body::Incoming,
    // response
    res_body_tx: futures::channel::mpsc::Sender<Result<http_body::Frame<Bytes>, anyhow::Error>>,
}

type IncomingMapErrBody =
    http_body_util::combinators::MapErr<hyper::body::Incoming, fn(hyper::Error) -> anyhow::Error>;

type BodyStreamNewMapToBytesStream = futures::stream::Map<
    http_body_util::BodyStream<IncomingMapErrBody>,
    fn(anyhow::Result<hyper::body::Frame<Bytes>>) -> anyhow::Result<Bytes>,
>;

type MultipartFieldMapToFrameStream = futures::stream::Map<
    mpart_async::server::MultipartField<BodyStreamNewMapToBytesStream, anyhow::Error>,
    fn(
        Result<Bytes, mpart_async::server::MultipartError>,
    ) -> anyhow::Result<hyper::body::Frame<Bytes>>,
>;

#[auto_enums::enum_derive(http_body1::Body)]
enum TransferRequestBody {
    Incoming(
        http_body_util::combinators::MapErr<
            hyper::body::Incoming,
            fn(hyper::Error) -> anyhow::Error,
        >,
    ),
    Multipart(http_body_util::StreamBody<MultipartFieldMapToFrameStream>),
    #[allow(dead_code)]
    Box(http_body_util::combinators::BoxBody<Bytes, anyhow::Error>),
}

type DataReceiverResponseBody = FinishDetectableBody<TransferRequestBody>;

struct DataReceiver {
    res_sender: futures::channel::oneshot::Sender<http::Response<DataReceiverResponseBody>>,
}

struct Pipe {
    data_sender: Option<DataSender>,
    data_receiver: Option<DataReceiver>,
}

impl Pipe {
    fn new() -> Self {
        Self {
            data_sender: None,
            data_receiver: None,
        }
    }
}

pub struct PipingServer {
    path_to_pipe: Arc<dashmap::DashMap<String, futures::lock::Mutex<Pipe>>>,
}

impl Clone for PipingServer {
    fn clone(&self) -> Self {
        PipingServer {
            path_to_pipe: Arc::clone(&self.path_to_pipe),
        }
    }
}

impl PipingServer {
    pub fn new() -> Self {
        PipingServer {
            path_to_pipe: Arc::new(dashmap::DashMap::new()),
        }
    }

    pub async fn handle(
        self,
        uses_https: bool,
        req: http::Request<hyper::body::Incoming>,
    ) -> anyhow::Result<http::Response<impl http_body::Body<Data = Bytes, Error = anyhow::Error>>>
    {
        seq_macro::seq!(N in 1..=2 {
            #[derive(Debug)]
            #[auto_enums::enum_derive(http_body1::Body)]
            enum BodyEnum<D, E, Full, Empty, #(B~N,)*> {
                #[allow(dead_code)]
                BoxBody(http_body_util::combinators::BoxBody<D, E>),
                FullBody(Full),
                EmptyBody(Empty),
                #(Body~N(B~N),)*
            }
        });

        let (req_parts, req_body) = req.into_parts();
        let path = req_parts.uri.path();

        log::info!(
            "{} {:} {:?}",
            req_parts.method,
            req_parts.uri,
            req_parts.version,
        );

        if req_parts.method == http::Method::GET || req_parts.method == http::Method::HEAD {
            match path {
                reserved_paths::INDEX => {
                    return Ok(http::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/html")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(BodyEnum::FullBody(full_body(&**dynamic_resources::INDEX)))
                        .unwrap());
                }
                reserved_paths::NO_SCRIPT => {
                    use base64::Engine as _;
                    let query_params = query_param_to_hash_map(req_parts.uri.query());
                    let style_nonce: String = {
                        let mut nonce_bytes = [0u8; 16];
                        getrandom::getrandom(&mut nonce_bytes).unwrap();
                        base64::engine::general_purpose::STANDARD.encode(nonce_bytes)
                    };
                    let html = dynamic_resources::no_script_html(&query_params, &style_nonce);
                    return Ok(http::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/html")
                        .header("Access-Control-Allow-Origin", "*")
                        .header(
                            "Content-Security-Policy",
                            format!("default-src 'none'; style-src 'nonce-{style_nonce}'"),
                        )
                        .body(BodyEnum::FullBody(full_body(html)))
                        .unwrap());
                }
                reserved_paths::VERSION => {
                    let version: &'static str = env!("CARGO_PKG_VERSION");
                    return Ok(http::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(BodyEnum::FullBody(full_body(format!("{version} (Rust)\n"))))
                        .unwrap());
                }
                reserved_paths::HELP => {
                    let host: &str = req_parts
                        .headers
                        .get("host")
                        .map(|h| h.to_str().unwrap())
                        .unwrap_or_else(|| "hostname");
                    let x_forwarded_proto_is_https =
                        if let Some(proto) = req_parts.headers.get("x-forwarded-proto") {
                            proto.to_str().unwrap().contains("https")
                        } else {
                            false
                        };
                    let schema = if uses_https || x_forwarded_proto_is_https {
                        "https"
                    } else {
                        "http"
                    };
                    let base_url = Url::parse(format!("{schema}://{host}").as_str())
                        .unwrap_or_else(|_| "http://hostname/".parse().unwrap());
                    let help = dynamic_resources::help(&base_url);
                    return Ok(http::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(BodyEnum::FullBody(full_body(help)))
                        .unwrap());
                }
                reserved_paths::FAVICON_ICO => {
                    return Ok(http::Response::builder()
                        .status(204)
                        .body(BodyEnum::EmptyBody(empty_body()))
                        .unwrap());
                }
                reserved_paths::ROBOTS_TXT => {
                    return Ok(http::Response::builder()
                        .status(404)
                        // explicit `content-length: 0`: https://github.com/hyperium/hyper/pull/2836
                        .header("Content-Length", 0)
                        .body(BodyEnum::EmptyBody(empty_body()))
                        .unwrap());
                }
                _ => {}
            }
        }

        match req_parts.method {
            http::Method::GET => {
                if let Some(value) = req_parts.headers.get("service-worker") {
                    if value == http::HeaderValue::from_static("script") {
                        // Reject Service Worker registration
                        return Ok(rejection_response(BodyEnum::FullBody(full_body(
                            "[ERROR] Service Worker registration is rejected.\n",
                        ))));
                    }
                }
                let query_params = query_param_to_hash_map(req_parts.uri.query());
                let Ok(n_receivers): Result<u32, _> = get_n_receivers_result(&query_params) else {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(
                        "[ERROR] Invalid \"n\" query parameter\n",
                    ))));
                };
                if n_receivers <= 0 {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(format!(
                        "[ERROR] n should > 0, but n = {n_receivers}.\n"
                    )))));
                }
                if n_receivers > 1 {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(
                        "[ERROR] n > 1 not supported yet.\n",
                    ))));
                }
                let pipe_mutex = self
                    .path_to_pipe
                    .entry(path.to_owned())
                    .or_insert_with(|| futures::lock::Mutex::new(Pipe::new()));
                let mut pipe_guard = pipe_mutex.lock().await;
                let receiver_connected: bool = (&pipe_guard.data_receiver).is_some();
                // If a receiver has been connected already
                if receiver_connected {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(format!(
                        "[ERROR] Another receiver has been connected on '{path}'.\n",
                    )))));
                }
                let (res_sender, res_receiver) = futures::channel::oneshot::channel::<
                    http::Response<DataReceiverResponseBody>,
                >();
                match pipe_guard.data_sender.take() {
                    // If sender is found
                    Some(mut data_sender) => {
                        data_sender
                            .res_body_tx
                            .send(Ok(http_body::Frame::data(Bytes::from(
                                "[INFO] A receiver was connected.\n",
                            ))))
                            .await
                            .unwrap();
                        transfer(path.to_string(), data_sender, DataReceiver { res_sender })
                            .await
                            .unwrap();
                    }
                    // If sender is not found
                    None => {
                        pipe_guard
                            .data_receiver
                            .replace(DataReceiver { res_sender });
                    }
                };
                drop(pipe_guard);
                drop(pipe_mutex);
                let (res_parts, res_body) = res_receiver.await?.into_parts();
                Ok(http::Response::from_parts(
                    res_parts,
                    BodyEnum::Body1(res_body),
                ))
            }
            http::Method::POST | http::Method::PUT => {
                if reserved_paths::VALUES.contains(&path) {
                    // Reject reserved path sending
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(Bytes::from(format!("[ERROR] Cannot send to the reserved path '{path}'. (e.g. '/mypath123')\n"))))));
                }
                // Notify that Content-Range is not supported
                // In the future, resumable upload using Content-Range might be supported
                // ref: https://github.com/httpwg/http-core/pull/653
                if req_parts.headers.contains_key("content-range") {
                    // Reject reserved path sending
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(
                        Bytes::from(format!(
                            "[ERROR] Content-Range is not supported for now in {}\n",
                            req_parts.method,
                        )),
                    ))));
                }
                let query_params = query_param_to_hash_map(req_parts.uri.query());
                let Ok(n_receivers): Result<u32, _> = get_n_receivers_result(&query_params) else {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(
                        Bytes::from("[ERROR] Invalid \"n\" query parameter\n"),
                    ))));
                };
                if n_receivers <= 0 {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(format!(
                        "[ERROR] n should > 0, but n = {n_receivers}.\n"
                    )))));
                }
                if n_receivers > 1 {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(
                        "[ERROR] n > 1 not supported yet.\n",
                    ))));
                }
                let pipe_mutex = self
                    .path_to_pipe
                    .entry(path.to_owned())
                    .or_insert_with(|| futures::lock::Mutex::new(Pipe::new()));
                let mut pipe_guard = pipe_mutex.lock().await;
                let sender_connected: bool = (&pipe_guard.data_sender).is_some();
                // If a sender has been connected already
                if sender_connected {
                    return Ok(rejection_response(BodyEnum::FullBody(full_body(format!(
                        "[ERROR] Another sender has been connected on '{path}'.\n"
                    )))));
                }

                let (mut res_body_tx, res_body_rx) = futures::channel::mpsc::channel::<
                    Result<http_body::Frame<Bytes>, anyhow::Error>,
                >(1);

                match pipe_guard.data_receiver.take() {
                    // If receiver is found
                    Some(data_receiver) => {
                        res_body_tx
                            .send(Ok(http_body::Frame::data(Bytes::from(
                                "[INFO] 1 receiver(s) has/have been connected.\n",
                            ))))
                            .await
                            .unwrap();
                        transfer(
                            path.to_string(),
                            DataSender {
                                req_headers: req_parts.headers,
                                req_body,
                                res_body_tx,
                            },
                            data_receiver,
                        )
                        .await
                        .unwrap();
                    }
                    // If receiver is not found
                    None => {
                        res_body_tx
                            .send(Ok(http_body::Frame::data(Bytes::from(
                                "[INFO] Waiting for 1 receiver(s)...\n",
                            ))))
                            .await
                            .unwrap();
                        pipe_guard.data_sender.replace(DataSender {
                            req_headers: req_parts.headers,
                            req_body,
                            res_body_tx,
                        });
                    }
                }
                drop(pipe_guard);
                drop(pipe_mutex);
                Ok(http::Response::builder()
                    .header("Content-Type", "text/plain")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(BodyEnum::Body2(http_body_util::StreamBody::new(
                        res_body_rx,
                    )))
                    .unwrap())
            }
            http::Method::OPTIONS => {
                // Response for Preflight request
                Ok(http::Response::builder()
                    .status(200)
                    .header("Access-Control-Allow-Origin", "*")
                    .header(
                        "Access-Control-Allow-Methods",
                        "GET, HEAD, POST, PUT, OPTIONS",
                    )
                    .header(
                        "Access-Control-Allow-Headers",
                        "Content-Type, Content-Disposition, X-Piping",
                    )
                    // Expose "Access-Control-Allow-Headers" for Web browser detecting X-Piping feature
                    .header(
                        "Access-Control-Expose-Headers",
                        "Access-Control-Allow-Headers",
                    )
                    // Private Network Access preflights: https://developer.chrome.com/blog/private-network-access-preflight/
                    .option_header(
                        "Access-Control-Allow-Private-Network",
                        if req_parts
                            .headers
                            .get("Access-Control-Request-Private-Network")
                            == Some(&http::header::HeaderValue::from_static("true"))
                        {
                            Some("true")
                        } else {
                            None
                        },
                    )
                    .header("Access-Control-Max-Age", 86400)
                    .header("Content-Length", 0)
                    .body(BodyEnum::EmptyBody(empty_body()))
                    .unwrap())
            }
            _ => {
                log::info!("Unsupported method: {}", req_parts.method);
                Ok(http::Response::builder()
                    .status(405)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(BodyEnum::FullBody(full_body(format!(
                        "[ERROR] Unsupported method: {}.\n",
                        req_parts.method
                    ))))
                    .unwrap())
            }
        }
    }
}

struct TransferRequest {
    content_type: Option<http::HeaderValue>,
    content_length: Option<http::HeaderValue>,
    content_disposition: Option<http::HeaderValue>,
    body: TransferRequestBody,
}

impl TransferRequest {
    #[inline]
    fn from_hyper_incoming(
        headers: &http::header::HeaderMap,
        body: hyper::body::Incoming,
    ) -> TransferRequest {
        TransferRequest {
            content_type: headers.get("content-type").cloned(),
            content_length: headers.get("content-length").cloned(),
            content_disposition: headers.get("content-disposition").cloned(),
            body: TransferRequestBody::Incoming(body.map_err(|e| e.into())),
        }
    }
}

async fn get_transfer_request(
    headers: &http::header::HeaderMap,
    body: hyper::body::Incoming,
) -> anyhow::Result<TransferRequest> {
    let Some(content_type) = headers.get("content-type") else {
        return Ok(TransferRequest::from_hyper_incoming(headers, body));
    };
    let mime_type_result: Result<mime::Mime, anyhow::Error> =
        (|| Ok(content_type.to_str()?.parse()?))();
    let Ok(mime_type): Result<mime::Mime, _> = mime_type_result else {
        return Ok(TransferRequest::from_hyper_incoming(headers, body));
    };
    if mime_type.essence_str() != "multipart/form-data" {
        return Ok(TransferRequest::from_hyper_incoming(headers, body));
    }
    let body: IncomingMapErrBody = body.map_err(|e| e.into());
    let boundary = mime_type
        .get_param("boundary")
        .map(|b| b.to_string())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "boundary not found"))?;

    let body_stream: BodyStreamNewMapToBytesStream = http_body_util::BodyStream::new(body).map(
        |result: Result<hyper::body::Frame<Bytes>, anyhow::Error>| {
            let b: Bytes = result?
                .into_data()
                .map_err(|_| anyhow!("failed to convert data for multipart"))?;
            Ok::<_, anyhow::Error>(b)
        },
    );
    let mut multipart_stream = mpart_async::server::MultipartStream::new(boundary, body_stream);

    while let Ok(Some(field)) = multipart_stream.try_next().await {
        // NOTE: Only first one is transferred
        let headers = field.headers().clone();
        let frame_stream: MultipartFieldMapToFrameStream = field.map(|result| {
            result
                .map(|b| hyper::body::Frame::data(b))
                .map_err(|e| e.into())
        });
        return Ok(TransferRequest {
            content_type: headers.get("content-type").cloned(),
            content_length: headers.get("content-length").cloned(),
            content_disposition: headers.get("content-disposition").cloned(),
            body: TransferRequestBody::Multipart(http_body_util::StreamBody::new(frame_stream)),
        });
    }
    anyhow::bail!("multipart error")
}

async fn transfer(
    path: String,
    data_sender: DataSender,
    data_receiver: DataReceiver,
) -> anyhow::Result<()> {
    let DataSender {
        req_headers: data_sender_req_headers,
        req_body: data_sender_req_body,
        res_body_tx: mut data_sender_res_body_tx,
    } = data_sender;
    log::info!("Transfer start: '{path}'");
    let transfer_request =
        get_transfer_request(&data_sender_req_headers, data_sender_req_body).await?;
    // The finish_waiter will tell when the body is finished
    let (finish_detectable_body, sender_req_body_finish_waiter) =
        finish_detectable_body(transfer_request.body);
    let x_piping = data_sender_req_headers.get_all("x-piping");
    let has_x_piping = data_sender_req_headers.contains_key("x-piping");
    // Create receiver's response
    let receiver_res = http::Response::builder()
        .option_header("Content-Type", transfer_request.content_type)
        .option_header("Content-Length", transfer_request.content_length)
        .option_header("Content-Disposition", transfer_request.content_disposition)
        .header_values("X-Piping", x_piping.into_iter().cloned())
        .header("Access-Control-Allow-Origin", "*")
        .option_header(
            "Access-Control-Expose-Headers",
            if has_x_piping { Some("X-Piping") } else { None },
        )
        .header("X-Robots-Tag", "none")
        .body(finish_detectable_body)
        .unwrap();
    // Return response to receiver
    data_receiver
        .res_sender
        .send(receiver_res)
        .map_err(|_| anyhow!("Failed to respond to receiver"))?;

    tokio::spawn(async move {
        data_sender_res_body_tx
            .send(Ok(http_body::Frame::data(Bytes::from(
                "[INFO] Start sending to 1 receiver(s)...\n",
            ))))
            .await
            .unwrap();
        // Wait for sender's request body finished
        if let Ok(_) = sender_req_body_finish_waiter.await {
            data_sender_res_body_tx
                .send(Ok(http_body::Frame::data(Bytes::from(
                    "[INFO] Sent successfully!\n",
                ))))
                .await
                .unwrap();
        } else {
            data_sender_res_body_tx
                .send(Ok(http_body::Frame::data(Bytes::from(
                    "[INFO] All receiver(s) was/were halfway disconnected.\n",
                ))))
                .await
                .unwrap();
        }
        log::info!("Transfer end: '{path}'");
    });
    return Ok(());
}

fn get_n_receivers_result(
    query_params: &HashMap<String, String>,
) -> Result<u32, std::num::ParseIntError> {
    return query_params
        .get("n")
        .map(|s| s.parse())
        .unwrap_or_else(|| Ok(1));
}

fn rejection_response<B>(body: B) -> http::Response<B> {
    http::Response::builder()
        .status(400)
        .header("Content-Type", "text/plain")
        .header("Access-Control-Allow-Origin", "*")
        .body(body)
        .unwrap()
}
