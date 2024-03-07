use crate::Stopper;
use event_listener::EventListener;
use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

/// A future that awaits this Stopper being stopped
#[derive(Debug)]
pub struct Stopped {
    stopper: Stopper,
    event_listener: EventListener,
}

impl From<Stopper> for Stopped {
    fn from(value: Stopper) -> Self {
        value.into_future()
    }
}

impl From<Stopped> for Stopper {
    fn from(value: Stopped) -> Self {
        value.stopper
    }
}

impl Future for Stopped {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            stopper,
            event_listener,
        } = &mut *self;
        loop {
            if stopper.is_stopped() {
                return Poll::Ready(());
            }

            match Pin::new(&mut *event_listener).poll(cx) {
                Poll::Ready(()) => continue,
                Poll::Pending => return Poll::Pending,
            };
        }
    }
}

impl IntoFuture for Stopper {
    type Output = ();
    type IntoFuture = Stopped;

    fn into_future(self) -> Self::IntoFuture {
        let event_listener = self.0.event.listen();
        Stopped {
            stopper: self,
            event_listener,
        }
    }
}
