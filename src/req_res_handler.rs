use core::future::Future;
use futures::channel::oneshot;
use futures::FutureExt;
use http::{Request, Response};
use hyper::Body;

// NOTE: futures::future::Map<..., oneshot::Receiver, ...> can be a Future
pub fn req_res_handler<Fut>(
    mut handler: impl FnMut(Request<Body>, oneshot::Sender<Response<Body>>) -> Fut,
) -> impl (FnMut(
    Request<Body>,
) -> futures::future::Map<
    futures::future::Join<Fut, oneshot::Receiver<Response<Body>>>,
    fn(
        ((), Result<Response<Body>, oneshot::Canceled>),
    ) -> Result<Response<Body>, oneshot::Canceled>,
>)
where
    Fut: Future<Output = ()>,
{
    move |req| {
        let (res_sender, res_receiver) = oneshot::channel::<Response<Body>>();
        futures::future::join(handler(req, res_sender), res_receiver).map(|(_, x)| x)
    }
}
