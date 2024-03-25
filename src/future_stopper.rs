use crate::Stopper;
use event_listener::EventListener;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    #[allow(missing_debug_implementations)]
    /// A wrapper that cancels the contained [`Future`] at an await point and returns None when
    /// the associated [`Stopper`] is stopped.
    pub struct FutureStopper<F> {
        #[pin]
        future: F,
        stopper: Stopper,
        event_listener: Option<EventListener>,
    }
}

impl<F: Future> FutureStopper<F> {
    pub(crate) fn new(stopper: &Stopper, future: F) -> Self {
        Self {
            future,
            stopper: stopper.clone(),
            event_listener: None,
        }
    }
}

impl<F: Future> Future for FutureStopper<F> {
    type Output = Option<<F as Future>::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let stopper = this.stopper;
        let event_listener = this.event_listener;
        let future = this.future;
        loop {
            if stopper.is_stopped_relaxed() {
                return Poll::Ready(None);
            }

            let listener = match event_listener {
                Some(listener) => listener,
                None => {
                    let listener = event_listener.insert(stopper.listener());
                    if stopper.is_stopped() {
                        return Poll::Ready(None);
                    }
                    listener
                }
            };

            match Pin::new(listener).poll(cx) {
                Poll::Ready(()) => {
                    *event_listener = None;
                }

                Poll::Pending if stopper.is_stopped_relaxed() => {
                    return Poll::Ready(None);
                }

                Poll::Pending => {
                    return future.poll(cx).map(Some);
                }
            };
        }
    }
}
