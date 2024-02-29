use core::convert::Infallible;
use core::convert::TryFrom;
use core::ops::Deref as _;
use core::pin::Pin;
use core::task::{Context, Poll};
use http_body_util::BodyExt as _;
use hyper::body::Bytes;
use pin_project_lite::pin_project;
use std::collections::HashMap;
use std::sync::Arc;

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
    pub struct FinishDetectableBody<B> {
        #[pin]
        body: B,
        finish_notifier: Option<futures::channel::oneshot::Sender<()>>,
    }
}

impl<B: http_body::Body> http_body::Body for FinishDetectableBody<B> {
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let mut this = self.project();
        match this.body.as_mut().poll_frame(cx) {
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

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.body.is_end_stream()
    }

    #[inline]
    fn size_hint(&self) -> http_body::SizeHint {
        self.body.size_hint()
    }
}

pub fn finish_detectable_body<B: http_body::Body>(
    body: B,
) -> (
    FinishDetectableBody<B>,
    futures::channel::oneshot::Receiver<()>,
) {
    let (finish_notifier, finish_waiter) = futures::channel::oneshot::channel::<()>();
    (
        FinishDetectableBody {
            body,
            finish_notifier: Some(finish_notifier),
        },
        finish_waiter,
    )
}

pub fn load_tls_config(
    cert_path: impl AsRef<std::path::Path>,
    key_path: impl AsRef<std::path::Path> + std::fmt::Display,
) -> anyhow::Result<rustls::ServerConfig> {
    let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(&mut std::fs::File::open(
        cert_path,
    )?))
    .collect::<Result<Vec<_>, _>>()?;
    let private_key = rustls_pemfile::private_key(&mut std::io::BufReader::new(
        &mut std::fs::File::open(key_path)?,
    ))?
    .expect("private key not found");
    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(config)
}

pub fn hot_reload_tls_cfg(
    tokio_handle: tokio::runtime::Handle,
    cert_path: impl AsRef<std::path::Path> + Send + Sync + 'static,
    key_path: impl AsRef<std::path::Path> + Send + Sync + std::fmt::Display + 'static,
) -> Arc<tokio::sync::RwLock<Arc<rustls::ServerConfig>>> {
    let cert_path = Arc::new(cert_path);
    let key_path = Arc::new(key_path);
    let tls_cfg_rwlock_arc = Arc::new(tokio::sync::RwLock::new(Arc::new(
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
                let received: Result<_, _> = match rx.recv() {
                    Ok(x) => x,
                    Err(e) => {
                        log::error!("Watch certificates error: {e:?}");
                        break;
                    }
                };
                let _event: notify::Event = match received {
                    Ok(x) => x,
                    Err(e) => {
                        log::error!("Watch certificates error: {e:?}");
                        continue;
                    }
                };
                let tls_cfg = match load_tls_config(cert_path.deref(), key_path.deref()) {
                    Ok(x) => x,
                    Err(e) => {
                        log::error!("Failed to load new certificates: {e:?}");
                        continue;
                    }
                };
                let tls_cfg_rwlock = tls_cfg_rwlock.clone();
                tokio_handle.spawn(async move {
                    *(tls_cfg_rwlock.write().await) = Arc::new(tls_cfg);
                    log::info!("Successfully new certificates loaded");
                });
            }
            Ok(())
        }
    });

    return tls_cfg_rwlock_arc;
}

pub fn query_param_to_hash_map(query: Option<&str>) -> HashMap<String, String> {
    return match query {
        Some(query) => serde_urlencoded::from_str::<HashMap<String, String>>(query)
            .unwrap_or_else(|_| HashMap::new()),
        None => HashMap::new(),
    };
}

#[inline]
pub fn full_body<B: Into<Bytes>, E>(
    b: B,
) -> http_body_util::combinators::MapErr<http_body_util::Full<Bytes>, fn(Infallible) -> E> {
    http_body_util::Full::new(b.into())
        .map_err(|_: Infallible| unreachable!("Error of Full::new() should be Infallible"))
}

#[inline]
pub fn empty_body<E>() -> impl http_body::Body<Data = Bytes, Error = E> {
    http_body_util::Empty::<Bytes>::new()
        .map_err(|_: Infallible| unreachable!("Error of Empty::new() should be Infallible"))
}
