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

pub struct FinishDetectableStream<S> {
    stream_pin: Pin<Box<S>>,
    finish_notifier: Option<oneshot::Sender<()>>,
}

impl<S: futures::stream::Stream> futures::stream::Stream for FinishDetectableStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().stream_pin.as_mut().poll_next(cx) {
            // If body is finished
            Poll::Ready(None) => {
                // Notify finish
                if let Some(notifier) = self.finish_notifier.take() {
                    notifier.send(()).unwrap();
                }
                Poll::Ready(None)
            }
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S> FinishDetectableStream<S> {
    pub fn new(stream: S, finish_notifier: oneshot::Sender<()>) -> FinishDetectableStream<S> {
        FinishDetectableStream {
            stream_pin: Box::pin(stream),
            finish_notifier: Some(finish_notifier),
        }
    }
}
