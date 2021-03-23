use crate::Stopper;
use futures_lite::Stream;
use pin_project::{pin_project, pinned_drop};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

#[pin_project(PinnedDrop)]
pub struct StreamStopper<S> {
    #[pin]
    stream: S,
    stopper: Stopper,
    waker_id: Option<usize>,
}

impl<S> Deref for StreamStopper<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<S: Clone> Clone for StreamStopper<S> {
    fn clone(&self) -> Self {
        StreamStopper {
            stream: self.stream.clone(),
            stopper: self.stopper.clone(),
            waker_id: None,
        }
    }
}

impl<S> DerefMut for StreamStopper<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl<S: Debug> Debug for StreamStopper<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.stream, f)
    }
}

impl<S: Stream> StreamStopper<S> {
    pub(crate) fn new(stopper: &Stopper, stream: S) -> Self {
        Self {
            stream,
            stopper: stopper.clone(),
            waker_id: None,
        }
    }
}

#[pinned_drop]
impl<S> PinnedDrop for StreamStopper<S> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        this.stopper.remove(this.waker_id);
    }
}

impl<S: Stream> Stream for StreamStopper<S> {
    type Item = <S as Stream>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if this.stopper.is_stopped() {
            Poll::Ready(None)
        } else {
            let inner_result = this.stream.poll_next(cx);

            if let Poll::Pending = inner_result {
                this.stopper.replace(this.waker_id, cx);
            } else {
                this.stopper.remove(this.waker_id);
            }

            inner_result
        }
    }
}
