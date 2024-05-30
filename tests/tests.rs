use futures_lite::{
    future::{self, poll_once},
    stream, StreamExt,
};
use std::future::{Future, IntoFuture};
use stopper::Stopper;
use test_harness::test;

#[cfg(not(all(loom, feature = "loom")))]
use std::thread::spawn;

#[cfg(all(loom, feature = "loom"))]
use loom::thread::spawn;

#[cfg(not(all(loom, feature = "loom")))]
#[track_caller]
fn harness<F, Fut>(test: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()> + Send,
{
    future::block_on(test());
}

#[cfg(all(loom, feature = "loom"))]
#[track_caller]
fn harness<F, Fut>(test: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send,
{
    loom::model(move || loom::future::block_on(test()));
}

#[test(harness)]
async fn future_stopper() {
    let stopper = Stopper::new();
    let future = stopper.stop_future(future::pending::<()>());
    spawn(move || {
        stopper.stop();
    });
    assert_eq!(future.await, None);
}

#[test(harness)]
async fn stream_stopper() {
    let stopper = Stopper::new();
    let mut stream = stopper.stop_stream(stream::repeat("infinite stream"));
    spawn(move || {
        stopper.stop();
    });

    while let Some(item) = stream.next().await {
        println!("{}", item);
        #[cfg(all(loom, feature = "loom"))]
        loom::thread::yield_now();
    }

    assert_eq!(poll_once(stream.next()).await, Some(None));
}

#[test(harness)]
async fn stopped() {
    let stopper = Stopper::new();
    let future = stopper.clone().into_future();
    assert!(!stopper.is_stopped());
    assert!(future::poll_once(stopper.clone().into_future())
        .await
        .is_none());

    spawn({
        let stopper = stopper.clone();
        move || stopper.stop()
    });

    future.await;
    assert!(stopper.is_stopped());
    assert!(future::poll_once(stopper.into_future()).await.is_some());
}
