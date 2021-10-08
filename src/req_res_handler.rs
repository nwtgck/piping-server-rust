use crate::util;
use core::future::Future;
use futures::channel::oneshot;
use futures::FutureExt;
use http::{Request, Response};
use hyper::Body;

// NOTE: futures::future::Map<..., oneshot::Receiver, ...> can be a Future
// NOTE: HE stands for Handler's Error
pub fn req_res_handler<Fut, HE>(
    mut handler: impl FnMut(Request<Body>, oneshot::Sender<Response<Body>>) -> Fut,
) -> impl (FnMut(
    Request<Body>,
) -> futures::future::Map<
    futures::future::Join<Fut, oneshot::Receiver<Response<Body>>>,
    fn(
        (Result<(), HE>, Result<Response<Body>, oneshot::Canceled>),
    ) -> Result<Response<Body>, std::io::Error>,
>)
where
    Fut: Future<Output = Result<(), HE>>,
    HE: std::fmt::Debug,
{
    move |req| {
        let (res_sender, res_receiver) = oneshot::channel::<Response<Body>>();
        futures::future::join(handler(req, res_sender), res_receiver).map(|(handler_result, x)| {
            match handler_result {
                Ok(_) => x.map_err(|e| util::make_io_error(format!("res_receiver error: {:?}", e))),
                Err(err) => {
                    log::error!("server error: {:?}", err);
                    Err(util::make_io_error(format!("server error: {:?}", err)))
                }
            }
        })
    }
}
