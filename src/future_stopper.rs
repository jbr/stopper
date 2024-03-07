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
    pub struct FutureStopper<F> {
        #[pin]
        future: F,
        stopper: Stopper,
        event_listener: EventListener,
    }
}

impl<F: Future> FutureStopper<F> {
    pub(crate) fn new(stopper: &Stopper, future: F) -> Self {
        Self {
            future,
            stopper: stopper.clone(),
            event_listener: stopper.0.event.listen(),
        }
    }
}

impl<F: Future> Future for FutureStopper<F> {
    type Output = Option<<F as Future>::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        loop {
            if this.stopper.is_stopped() {
                return Poll::Ready(None);
            }

            match Pin::new(&mut *this.event_listener).poll(cx) {
                Poll::Ready(()) => continue,
                Poll::Pending => {
                    return match this.future.poll(cx) {
                        Poll::Ready(output) => Poll::Ready(Some(output)),
                        Poll::Pending => Poll::Pending,
                    }
                }
            };
        }
    }
}
