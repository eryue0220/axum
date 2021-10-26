//! Future types.

use crate::{body::BoxBody, clone_box_service::CloneBoxService};
use futures_util::future::Either;
use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    convert::Infallible,
    future::{ready, Future},
    pin::Pin,
    task::{Context, Poll},
};
use tower::util::Oneshot;
use tower_service::Service;

pub use super::method_not_allowed::MethodNotAllowedFuture;

opaque_future! {
    /// Response future for [`Router`](super::Router).
    pub type RouterFuture<B> =
        futures_util::future::Either<
            Oneshot<super::Route<B>, Request<B>>,
            std::future::Ready<Result<Response<BoxBody>, Infallible>>,
        >;
}

impl<B> RouterFuture<B> {
    pub(super) fn from_oneshot(future: Oneshot<super::Route<B>, Request<B>>) -> Self {
        Self::new(Either::Left(future))
    }

    pub(super) fn from_response(response: Response<BoxBody>) -> Self {
        RouterFuture::new(Either::Right(ready(Ok(response))))
    }
}

pin_project! {
    /// Response future for [`Route`](super::Route).
    pub struct RouteFuture<B> {
        #[pin]
        future: Oneshot<
            CloneBoxService<Request<B>, Response<BoxBody>, Infallible>,
            Request<B>,
        >
    }
}

impl<B> RouteFuture<B> {
    pub(crate) fn new(
        future: Oneshot<CloneBoxService<Request<B>, Response<BoxBody>, Infallible>, Request<B>>,
    ) -> Self {
        RouteFuture { future }
    }
}

impl<B> Future for RouteFuture<B> {
    type Output = Result<Response<BoxBody>, Infallible>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}

pin_project! {
    /// The response future for [`Nested`](super::Nested).
    #[derive(Debug)]
    pub(crate) struct NestedFuture<S, B>
    where
        S: Service<Request<B>>,
    {
        #[pin]
        pub(super) inner: Oneshot<S, Request<B>>
    }
}

impl<S, B> Future for NestedFuture<S, B>
where
    S: Service<Request<B>, Response = Response<BoxBody>, Error = Infallible>,
    B: Send + Sync + 'static,
{
    type Output = Result<Response<BoxBody>, Infallible>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

opaque_future! {
    /// Response future from [`MakeRouteService`] services.
    pub type MakeRouteServiceFuture<S> =
        std::future::Ready<Result<S, Infallible>>;
}
