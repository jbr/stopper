use crate::Stopper;
use pin_project::{pin_project, pinned_drop};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[pin_project(PinnedDrop)]
pub struct FutureStopper<F> {
    #[pin]
    future: F,
    stopper: Stopper,
    waker_id: Option<usize>,
}

impl<F: Future> FutureStopper<F> {
    pub(crate) fn new(stopper: &Stopper, future: F) -> Self {
        Self {
            future,
            stopper: stopper.clone(),
            waker_id: None,
        }
    }
}

#[pinned_drop]
impl<F> PinnedDrop for FutureStopper<F> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        this.stopper.remove(this.waker_id);
    }
}

impl<F: Future> Future for FutureStopper<F> {
    type Output = Option<<F as Future>::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if this.stopper.is_stopped() {
            Poll::Ready(None)
        } else {
            match this.future.poll(cx) {
                Poll::Ready(t) => {
                    this.stopper.remove(this.waker_id);
                    Poll::Ready(Some(t))
                }

                Poll::Pending => {
                    this.stopper.replace(this.waker_id, cx);
                    Poll::Pending
                }
            }
        }
    }
}
