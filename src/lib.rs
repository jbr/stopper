use futures_lite::{Future, Stream};
use pin_project::{pin_project, pinned_drop};
use std::task::Context;
use std::{
    pin::Pin,
    sync::{Arc, RwLock},
    task::Poll,
};
use waker_set::WakerSet;

pub struct StopperInner {
    stopped: bool,
    waker: WakerSet,
}

#[derive(Clone)]
pub struct Stopper(Arc<RwLock<StopperInner>>);

impl std::fmt::Debug for Stopper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lock = self.0.read().unwrap();
        f.debug_struct("Stopper")
            .field("stopped", &lock.stopped)
            .finish()
    }
}

impl Stopper {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(StopperInner {
            stopped: false,
            waker: WakerSet::new(),
        })))
    }

    pub fn stop(&self) {
        let mut lock = self.0.write().unwrap();
        lock.stopped = true;
        lock.waker.notify_all();
    }

    pub fn stop_stream<S: Stream>(&self, stream: S) -> StreamStopper<S> {
        StreamStopper {
            stopper: self.clone(),
            waker_id: None,
            stream,
        }
    }

    pub fn insert(&self, cx: &Context) -> usize {
        self.0.write().unwrap().waker.insert(cx)
    }

    pub fn replace(&self, waker_id: &mut Option<usize>, cx: &Context) {
        if let Some(id) = waker_id {
            self.remove_waker(*id);
        }
        waker_id.replace(self.insert(&cx));
    }

    pub fn is_stopped(&self) -> bool {
        self.0.read().unwrap().stopped
    }

    pub fn remove_waker(&self, id: usize) {
        self.0.write().unwrap().waker.cancel(id);
    }

    pub fn stop_future<F: Future>(&self, future: F) -> FutureStopper<F> {
        FutureStopper {
            stopper: self.clone(),
            waker_id: None,
            future,
        }
    }
}

#[pin_project(PinnedDrop)]
pub struct FutureStopper<F> {
    stopper: Stopper,
    waker_id: Option<usize>,
    #[pin]
    future: F,
}

#[pinned_drop]
impl<F> PinnedDrop for FutureStopper<F> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if let Some(id) = this.waker_id {
            this.stopper.remove_waker(*id);
        }
    }
}

impl<F: Future> Future for FutureStopper<F> {
    type Output = Option<<F as Future>::Output>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let lock = this.stopper.0.write().unwrap();

        if lock.stopped {
            Poll::Ready(None)
        } else {
            match this.future.poll(cx) {
                Poll::Ready(t) => Poll::Ready(Some(t)),
                Poll::Pending => {
                    if let Some(id) = this.waker_id {
                        lock.waker.cancel(*id);
                    }
                    this.waker_id.replace(lock.waker.insert(&cx));
                    Poll::Pending
                }
            }
        }
    }
}

#[pin_project(PinnedDrop)]
pub struct StreamStopper<S> {
    #[pin]
    stream: S,
    stopper: Stopper,
    waker_id: Option<usize>,
}

#[pinned_drop]
impl<S> PinnedDrop for StreamStopper<S> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if let Some(id) = this.waker_id {
            this.stopper.0.write().unwrap().waker.cancel(*id);
        }
    }
}

impl<T, S> Stream for StreamStopper<S>
where
    S: Stream<Item = T>,
{
    type Item = T;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let lock = this.stopper.0.write().unwrap();
        if lock.stopped {
            Poll::Ready(None)
        } else {
            match this.stream.poll_next(cx) {
                Poll::Pending => {
                    if let Some(id) = this.waker_id {
                        lock.waker.cancel(*id);
                    }
                    this.waker_id.replace(lock.waker.insert(cx));
                    Poll::Pending
                }
                x => x,
            }
        }
    }
}
