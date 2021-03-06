#![forbid(unsafe_code, future_incompatible)]
#![deny(
    missing_debug_implementations,
    nonstandard_style,
    missing_copy_implementations,
    unused_qualifications
)]
//! # Stopper
//!
//! The primary type for this crate is [`Stopper`], which provides a
//! synchronized mechanism for canceling Futures and Streams.

use futures_lite::Stream;
use std::{
    fmt::{Debug, Formatter, Result},
    future::Future,
    sync::{Arc, RwLock},
    task::Context,
};
use waker_set::WakerSet;

mod stream_stopper;
pub use stream_stopper::StreamStopper;

mod future_stopper;
pub use future_stopper::FutureStopper;

/// This struct provides a synchronized mechanism for canceling
/// Futures and Streams.
#[derive(Clone)]
pub struct Stopper(Arc<RwLock<StopperInner>>);

impl Stopper {
    /// Initialize a stopper that is not yet stopped and that has zero
    /// registered wakers. Any clone of this stopper represents the
    /// same internal state. This is identical to Stopper::default()
    pub fn new() -> Self {
        Self::default()
    }

    /// Stop all futures and streams that have been registered to this
    /// Stopper or any clone representing the same initial stopper.
    ///
    pub fn stop(&self) {
        let mut lock = self.0.write().unwrap();
        if !lock.stopped {
            lock.stopped = true;
            lock.waker.notify_all();
        }
    }

    /// Returns whether this stopper (or any clone of it) has been
    /// stopped.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let stopper = stopper::Stopper::new();
    /// assert!(!stopper.is_stopped());
    /// stopper.stop();
    /// assert!(stopper.is_stopped());
    /// ```
    pub fn is_stopped(&self) -> bool {
        self.0.read().unwrap().stopped
    }

    /// This function returns a new stream which will poll None
    /// (indicating a completed stream) when this Stopper has been
    /// stopped. The Stream's Item is unchanged.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # fn main() { async_io::block_on(async {
    /// use futures_lite::StreamExt;
    /// let stopper = stopper::Stopper::new();
    /// let mut stream = stopper.stop_stream(futures_lite::stream::repeat("infinite stream"));
    ///
    /// std::thread::spawn(move || {
    ///     std::thread::sleep(std::time::Duration::from_secs(1));
    ///     stopper.stop();
    /// });
    ///
    /// while let Some(item) = stream.next().await {
    ///     println!("{}", item);
    /// }
    /// # }) }
    /// ```
    pub fn stop_stream<S: Stream>(&self, stream: S) -> StreamStopper<S> {
        StreamStopper::new(&self, stream)
    }

    /// This function returns a Future which wraps the provided future
    /// and stops it when this Stopper has been stopped. Note that the
    /// Output of the returned future is wrapped with an Option. If
    /// the future resolves to None, that indicates that it was
    /// stopped instead of polling to completion.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # fn main() { async_io::block_on(async {
    /// let stopper = stopper::Stopper::new();
    /// let mut future = stopper.stop_future(std::future::pending::<()>());
    ///
    /// std::thread::spawn(move || {
    ///     std::thread::sleep(std::time::Duration::from_secs(1));
    ///     stopper.stop();
    /// });
    ///
    /// assert_eq!(future.await, None);
    ///
    /// # }) }
    /// ```
    pub fn stop_future<F: Future>(&self, future: F) -> FutureStopper<F> {
        FutureStopper::new(&self, future)
    }

    pub(crate) fn insert(&self, cx: &Context) -> usize {
        self.0.write().unwrap().waker.insert(cx)
    }

    pub(crate) fn replace(&self, waker_id: &mut Option<usize>, cx: &Context) {
        if let Some(id) = waker_id {
            self.remove_waker(*id);
        }

        waker_id.replace(self.insert(&cx));
    }

    pub(crate) fn remove(&self, waker_id: &mut Option<usize>) {
        if let Some(id) = waker_id.take() {
            self.remove_waker(id);
        }
    }

    pub(crate) fn remove_waker(&self, id: usize) {
        self.0.write().unwrap().waker.cancel(id);
    }
}

impl Default for Stopper {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(StopperInner {
            stopped: false,
            waker: WakerSet::new(),
        })))
    }
}

impl Debug for Stopper {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let lock = self.0.read().unwrap();
        f.debug_struct("Stopper")
            .field("stopped", &lock.stopped)
            .finish()
    }
}

struct StopperInner {
    stopped: bool,
    waker: WakerSet,
}
