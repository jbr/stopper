#![forbid(unsafe_code, future_incompatible)]
#![deny(
    missing_debug_implementations,
    nonstandard_style,
    missing_copy_implementations,
    unused_qualifications,
    missing_docs,
    rustdoc::missing_crate_level_docs
)]
#![warn(clippy::pedantic)]
//! # Stopper
//!
//! The primary type for this crate is [`Stopper`], which provides a
//! synchronized mechanism for canceling Futures and Streams.

use event_listener::{Event, EventListener, IntoNotification};
use futures_lite::Stream;
use std::{
    fmt::{Debug, Formatter, Result},
    future::Future,
};

#[cfg(all(loom, feature = "loom"))]
use loom::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
#[cfg(not(all(loom, feature = "loom")))]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod stopped;
pub use stopped::Stopped;

mod stream_stopper;
pub use stream_stopper::StreamStopper;

mod future_stopper;
pub use future_stopper::FutureStopper;

/// This struct provides a synchronized mechanism for canceling
/// Futures and Streams.
///
/// Stoppers are cheap to clone.
///
/// A clone of the Stopper can be awaited and will be pending until the Stopper is stopped. If the
/// Stopper is stopped before it is awaited, it will be ready immediately.
#[derive(Clone)]
pub struct Stopper(Arc<StopperInner>);

impl From<StopperInner> for Stopper {
    fn from(value: StopperInner) -> Self {
        Self(Arc::new(value))
    }
}

impl Stopper {
    /// Initialize a stopper that is not yet stopped and that has zero
    /// registered wakers. Any clone of this stopper represents the
    /// same internal state. This is identical to `Stopper::default()`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stop all futures and streams that have been registered to this
    /// Stopper or any clone representing the same initial stopper.
    ///
    pub fn stop(&self) {
        if !self.0.stopped.swap(true, Ordering::SeqCst) {
            self.0.event.notify(usize::MAX.relaxed());
        }
    }

    /// Returns whether this stopper (or any clone of it) has been
    /// stopped.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.0.stopped.load(Ordering::SeqCst)
    }

    pub(crate) fn is_stopped_relaxed(&self) -> bool {
        self.0.stopped.load(Ordering::Relaxed)
    }

    pub(crate) fn listener(&self) -> EventListener {
        self.0.event.listen()
    }

    /// This function returns a new stream which will poll None
    /// (indicating a completed stream) when this Stopper has been
    /// stopped. The Stream's Item is unchanged.
    pub fn stop_stream<S: Stream>(&self, stream: S) -> StreamStopper<S> {
        StreamStopper::new(self, stream)
    }

    /// This function returns a Future which wraps the provided future
    /// and stops it when this Stopper has been stopped. Note that the
    /// Output of the returned future is wrapped with an Option. If
    /// the future resolves to None, that indicates that it was
    /// stopped instead of polling to completion.
    pub fn stop_future<F: Future>(&self, future: F) -> FutureStopper<F> {
        FutureStopper::new(self, future)
    }
}

impl Default for Stopper {
    fn default() -> Self {
        Self::from(StopperInner {
            stopped: false.into(),
            event: Event::new(),
        })
    }
}

impl Debug for Stopper {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Stopper")
            .field("stopped", &self.is_stopped())
            .finish()
    }
}

struct StopperInner {
    stopped: AtomicBool,
    event: Event,
}
