use core::convert::TryFrom;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::channel::oneshot;
use futures::ready;
use pin_project_lite::pin_project;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub trait OptionHeaderBuilder {
    // Add optional header
    fn option_header<K, V>(self, key: K, value_opt: Option<V>) -> Self
    where
        http::header::HeaderName: TryFrom<K>,
        <http::header::HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        http::header::HeaderValue: TryFrom<V>,
        <http::header::HeaderValue as TryFrom<V>>::Error: Into<http::Error>;
}

impl OptionHeaderBuilder for http::response::Builder {
    // Add optional header
    fn option_header<K, V>(self, key: K, value_opt: Option<V>) -> Self
    where
        http::header::HeaderName: TryFrom<K>,
        <http::header::HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        http::header::HeaderValue: TryFrom<V>,
        <http::header::HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        if let Some(value) = value_opt {
            self.header(key, value)
        } else {
            self
        }
    }
}

pub trait HeaderValuesBuilder {
    fn header_values<K, I>(self, key: K, values: I) -> Self
    where
        K: Clone + http::header::IntoHeaderName,
        http::header::HeaderName: TryFrom<K>,
        <http::header::HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        I: IntoIterator<Item = http::header::HeaderValue>;
}

impl HeaderValuesBuilder for http::response::Builder {
    fn header_values<K, I>(mut self, key: K, values: I) -> Self
    where
        K: Clone + http::header::IntoHeaderName,
        http::header::HeaderName: TryFrom<K>,
        <http::header::HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        I: IntoIterator<Item = http::header::HeaderValue>,
    {
        if let Some(headers) = self.headers_mut() {
            for value in values {
                headers.append(key.clone(), value);
            }
        }
        self
    }
}

pin_project! {
    pub struct FinishDetectableStream<S> {
        #[pin]
        stream_pin: S,
        finish_notifier: Option<oneshot::Sender<()>>,
    }
}

impl<S: futures::stream::Stream> futures::stream::Stream for FinishDetectableStream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        match this.stream_pin.as_mut().poll_next(cx) {
            // If body is finished
            Poll::Ready(None) => {
                // Notify finish
                if let Some(notifier) = this.finish_notifier.take() {
                    notifier.send(()).unwrap();
                }
                Poll::Ready(None)
            }
            poll => poll,
        }
    }
}

pub fn finish_detectable_stream<S>(
    stream: S,
) -> (FinishDetectableStream<S>, oneshot::Receiver<()>) {
    let (finish_notifier, finish_waiter) = oneshot::channel::<()>();
    (
        FinishDetectableStream {
            stream_pin: stream,
            finish_notifier: Some(finish_notifier),
        },
        finish_waiter,
    )
}

pub fn make_io_error(err: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err)
}

fn read_private_keys(rd: &mut dyn std::io::BufRead) -> Result<Vec<Vec<u8>>, std::io::Error> {
    let mut keys = Vec::<Vec<u8>>::new();

    loop {
        match rustls_pemfile::read_one(rd)? {
            None => return Ok(keys),
            Some(
                rustls_pemfile::Item::RSAKey(key)
                | rustls_pemfile::Item::PKCS8Key(key)
                | rustls_pemfile::Item::ECKey(key),
            ) => keys.push(key),
            _ => {}
        };
    }
}

// (base: https://github.com/ctz/hyper-rustls/blob/5f073724f7b5eee3a2d72f0a86094fc2718b51cd/examples/server.rs)
pub fn load_tls_config(
    cert_path: impl AsRef<std::path::Path>,
    key_path: impl AsRef<std::path::Path> + std::fmt::Display,
) -> std::io::Result<rustls::ServerConfig> {
    // Load public certificate.
    let mut cert_reader = std::io::BufReader::new(std::fs::File::open(cert_path)?);
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .map_err(|_| make_io_error("unable to load certificate".to_owned()))?;
    // Load private key.
    let mut key_reader = std::io::BufReader::new(std::fs::File::open(key_path)?);
    // Load and return a single private key.
    let mut private_keys = read_private_keys(&mut key_reader)
        .map_err(|_| make_io_error("unable to load private key".to_owned()))?;
    let key = if private_keys.len() > 0 {
        Ok(private_keys.remove(0))
    } else {
        Err(make_io_error("failed to get private key".to_owned()))
    }?;
    let certificates: Vec<rustls::Certificate> =
        certs.into_iter().map(rustls::Certificate).collect();
    let mut cfg = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates, rustls::PrivateKey(key))
        .map_err(|_| make_io_error("failed to create ServerConfig".to_owned()))?;
    // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
    cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(cfg)
}

pub fn hot_reload_tls_cfg(
    cert_path: impl AsRef<std::path::Path> + Send + Sync + 'static,
    key_path: impl AsRef<std::path::Path> + Send + Sync + std::fmt::Display + 'static,
) -> Arc<RwLock<Arc<rustls::ServerConfig>>> {
    let cert_path = Arc::new(cert_path);
    let key_path = Arc::new(key_path);
    let tls_cfg_rwlock_arc = Arc::new(RwLock::new(Arc::new(
        load_tls_config(cert_path.deref(), key_path.deref()).unwrap(),
    )));

    // NOTE: tokio::spawn() blocks servers in some environment because of `loop {}`
    std::thread::spawn::<_, Result<(), notify::Error>>({
        let tls_cfg_rwlock = tls_cfg_rwlock_arc.clone();
        move || {
            use notify::Watcher;
            let (tx, rx) = std::sync::mpsc::channel();

            let mut watcher: notify::RecommendedWatcher = notify::Watcher::new(
                tx,
                notify::Config::default().with_poll_interval(std::time::Duration::from_secs(5)),
            )?;

            watcher.watch(
                cert_path.deref().as_ref(),
                notify::RecursiveMode::NonRecursive,
            )?;
            watcher.watch(
                key_path.deref().as_ref(),
                notify::RecursiveMode::NonRecursive,
            )?;

            loop {
                match rx.recv() {
                    Ok(_event) => match load_tls_config(cert_path.deref(), key_path.deref()) {
                        Ok(tls_cfg) => {
                            *(tls_cfg_rwlock.clone().write().unwrap()) = Arc::new(tls_cfg);
                            log::info!("Successfully new certificates loaded");
                        }
                        Err(e) => log::error!("Failed to load new certificates: {:?}", e),
                    },
                    Err(e) => log::error!("Watch certificates error: {:?}", e),
                }
            }
        }
    });

    return tls_cfg_rwlock_arc;
}

pin_project! {
    pub struct HyperAcceptor<S> {
        #[pin]
        pub acceptor: S,
    }
}

impl<S> hyper::server::accept::Accept for HyperAcceptor<S>
where
    S: futures::stream::Stream<Item = Result<TlsStream<TcpStream>, std::io::Error>>,
{
    type Conn = TlsStream<TcpStream>;
    type Error = std::io::Error;

    fn poll_accept(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        self.project().acceptor.as_mut().poll_next(cx)
    }
}

// (base: https://github.com/tokio-rs/tokio/blob/tokio-0.2.22/tokio/src/net/tcp/incoming.rs)
/// Stream returned by the `TcpListener::incoming` function representing the
/// stream of sockets received from a listener.
#[must_use = "streams do nothing unless polled"]
#[derive(Debug)]
pub struct TokioIncoming<'a> {
    inner: &'a mut tokio::net::TcpListener,
}

impl TokioIncoming<'_> {
    pub fn new(listener: &mut tokio::net::TcpListener) -> TokioIncoming<'_> {
        TokioIncoming { inner: listener }
    }
}

impl futures::stream::Stream for TokioIncoming<'_> {
    type Item = tokio::io::Result<TcpStream>;

    #[allow(unused_mut)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (socket, _) = ready!(self.inner.poll_accept(cx))?;
        Poll::Ready(Some(Ok(socket)))
    }
}

pin_project! {
    pub struct One<T> {
        value: Option<T>,
    }
}

impl<T> One<T> {
    fn new(x: T) -> Self {
        One { value: Some(x) }
    }
}

impl<T> futures::stream::Stream for One<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.project().value.take())
    }
}

#[inline]
pub fn one_stream<T>(x: T) -> One<T> {
    One::new(x)
}

pub fn query_param_to_hash_map(query: Option<&str>) -> HashMap<String, String> {
    return match query {
        Some(query) => serde_urlencoded::from_str::<HashMap<String, String>>(query)
            .unwrap_or_else(|_| HashMap::new()),
        None => HashMap::new(),
    };
}
