use futures::channel::oneshot;
use futures::FutureExt;
use hyper::{Body, Request, Response};
use std::future::Future;


// NOTE: futures::future::Then<..., oneshot::Receiver, ...> can be a Future
pub fn req_res_handler<F, Fut>(
    mut handler: F,
) -> impl (FnMut(
    Request<Body>,
) -> futures::future::Then<
    Fut,
    oneshot::Receiver<Response<Body>>,
    Box<dyn FnOnce((),) -> oneshot::Receiver<Response<Body>> + Send>,
>)
where
    F: FnMut(Request<Body>, oneshot::Sender<Response<Body>>) -> Fut,
    Fut: Future<Output = ()>,
{
    move |req| {
        let (res_sender, res_receiver) = oneshot::channel::<Response<Body>>();
        handler(req, res_sender).then(Box::new(move |_| {
            res_receiver
        }))
    }
}
