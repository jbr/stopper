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
        event_listener: Option<EventListener>
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
            event_listener: None,
        }
    }
}

impl<S: Stream> Stream for StreamStopper<S> {
    type Item = <S as Stream>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let stopper = this.stopper;
        let event_listener = this.event_listener;
        let stream = this.stream;

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
                    return stream.poll_next(cx);
                }
            };
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
