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
    event_listener: Pin<Box<EventListener>>,
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

            if event_listener.is_listening() {
                match event_listener.as_mut().poll(cx) {
                    Poll::Ready(()) => continue,
                    Poll::Pending => return Poll::Pending,
                };
            } else {
                event_listener.as_mut().listen(&stopper.0.event);
            }
        }
    }
}

impl IntoFuture for Stopper {
    type Output = ();
    type IntoFuture = Stopped;

    fn into_future(self) -> Self::IntoFuture {
        Stopped {
            stopper: self,
            event_listener: Box::pin(EventListener::new()),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        thread::{sleep, spawn},
        time::Duration,
    };

    use super::*;
    use futures_lite::future::{block_on, poll_once};

    #[test]
    fn stopped() {
        block_on(async {
            let stopper = Stopper::new();
            let future = stopper.clone().into_future();
            spawn({
                let stopper = stopper.clone();
                move || {
                    sleep(Duration::from_secs(2));
                    stopper.stop();
                }
            });
            assert!(!stopper.is_stopped());
            assert!(poll_once(stopper.clone().into_future()).await.is_none());
            future.await;
            assert!(stopper.is_stopped());
            assert!(poll_once(stopper.into_future()).await.is_some());
        });
    }
}
