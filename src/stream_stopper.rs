use crate::Stopper;
use event_listener::EventListener;
use futures_lite::Stream;
use std::{
    fmt::{self, Debug, Formatter},
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

pin_project_lite::pin_project! {
    /// A wrapper type that stops a [`Stream`] when the associated [`Stopper`] is stopped
    pub struct StreamStopper<S> {
        #[pin]
        stream: S,
        stopper: Stopper,
        event_listener: EventListener
    }
}

impl<S: Clone + Stream> Clone for StreamStopper<S> {
    fn clone(&self) -> Self {
        Self::new(&self.stopper, self.stream.clone())
    }
}

impl<S> Deref for StreamStopper<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<S> DerefMut for StreamStopper<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl<S: Debug> Debug for StreamStopper<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.stream, f)
    }
}

impl<S: Stream> StreamStopper<S> {
    pub(crate) fn new(stopper: &Stopper, stream: S) -> Self {
        Self {
            stream,
            stopper: stopper.clone(),
            event_listener: stopper.0.event.listen(),
        }
    }
}

impl<S: Stream> Stream for StreamStopper<S> {
    type Item = <S as Stream>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        loop {
            if this.stopper.is_stopped() {
                return Poll::Ready(None);
            }

            match Pin::new(&mut *this.event_listener).poll(cx) {
                Poll::Ready(()) => continue,
                Poll::Pending => return this.stream.poll_next(cx),
            };
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
