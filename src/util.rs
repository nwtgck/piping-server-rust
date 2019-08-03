use hyper::{Body, Chunk};
use futures::Async;
use futures::sync::oneshot;

pub trait OptionHeaderBuilder {
    // Add optional header
    fn option_header<K, V>(&mut self, key: K, value_opt: Option<V>) -> &mut Self
        where http::header::HeaderName: http::HttpTryFrom<K>,
              http::header::HeaderValue: http::HttpTryFrom<V>;
}

impl OptionHeaderBuilder for http::response::Builder {
    // Add optional header
    fn option_header<K, V>(&mut self, key: K, value_opt: Option<V>) -> &mut Self
        where http::header::HeaderName: http::HttpTryFrom<K>,
              http::header::HeaderValue: http::HttpTryFrom<V> {
        if let Some(value) = value_opt {
            self.header(key, value)
        } else {
            self

        }
    }
}


pub struct FinishDetectableBody {
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
    pub fn new(body: Body, finish_notifier: oneshot::Sender<()>) -> FinishDetectableBody {
        FinishDetectableBody {
            body,
            finish_notifier: Some(finish_notifier)
        }
    }
}
