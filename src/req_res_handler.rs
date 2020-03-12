use futures::channel::oneshot;
use futures::FutureExt;
use hyper::{Body, Request, Response};
use std::future::Future;

pub struct HoldOneReturnThatClosure<T> {
    value: T,
}

impl<T> std::ops::FnOnce<((),)> for HoldOneReturnThatClosure<T> {
    type Output = T;
    extern "rust-call" fn call_once(self, _args: ((),)) -> Self::Output {
        self.value
    }
}

// NOTE: futures::future::Then<..., oneshot::Receiver, ...> can be a Future
pub fn req_res_handler<F, Fut>(
    mut handler: F,
) -> impl (FnMut(
    Request<Body>,
) -> futures::future::Then<
    Fut,
    oneshot::Receiver<Response<Body>>,
    HoldOneReturnThatClosure<oneshot::Receiver<Response<Body>>>,
>)
where
    F: FnMut(Request<Body>, oneshot::Sender<Response<Body>>) -> Fut,
    Fut: Future<Output = ()>,
{
    move |req| {
        let (res_sender, res_receiver) = oneshot::channel::<Response<Body>>();
        handler(req, res_sender).then(HoldOneReturnThatClosure {
            value: res_receiver,
        })
    }
}
