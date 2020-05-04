use futures::channel::oneshot;
use futures::task::{Context, Poll};
use hyper::body::Body;
use hyper::body::Bytes;
use std::convert::TryFrom;
use std::pin::Pin;

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

pub struct FinishDetectableBody {
    body_pin: Pin<Box<Body>>,
    finish_notifier: Option<oneshot::Sender<()>>,
}

impl futures::stream::Stream for FinishDetectableBody {
    type Item = Result<Bytes, http::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().body_pin.as_mut().poll_next(cx) {
            // If body is finished
            Poll::Ready(None) => {
                // Notify finish
                if let Some(notifier) = self.as_mut().finish_notifier.take() {
                    notifier.send(()).unwrap();
                }
                Poll::Ready(None)
            }
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(Some(Err(_))) => Poll::Ready(Some(Ok(Bytes::from("")))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl FinishDetectableBody {
    pub fn new(body: Body, finish_notifier: oneshot::Sender<()>) -> FinishDetectableBody {
        FinishDetectableBody {
            body_pin: Box::pin(body),
            finish_notifier: Some(finish_notifier),
        }
    }
}
