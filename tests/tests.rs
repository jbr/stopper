use futures_lite::{future, stream, StreamExt};
use std::{
    future::{Future, IntoFuture},
    process::Termination,
    thread::{sleep, spawn},
    time::Duration,
};
use stopper::Stopper;
use test_harness::test;

fn harness<F, Fut, O>(test: F) -> O
where
    F: FnOnce() -> Fut,
    O: Termination,
    Fut: Future<Output = O> + Send,
{
    future::block_on(test())
}

#[test(harness)]
async fn future_stopper() {
    let stopper = Stopper::new();
    let future = stopper.stop_future(future::pending::<()>());

    spawn(move || {
        sleep(Duration::from_secs(1));
        stopper.stop();
    });

    assert_eq!(future.await, None);
}

#[test(harness)]
async fn stream_stopper() {
    let stopper = Stopper::new();
    let mut stream = stopper.stop_stream(stream::repeat("infinite stream"));

    spawn(move || {
        sleep(Duration::from_secs(1));
        stopper.stop();
    });

    while let Some(item) = stream.next().await {
        println!("{}", item);
    }
}

#[test(harness)]
async fn stopped() {
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
    assert!(future::poll_once(stopper.clone().into_future())
        .await
        .is_none());
    future.await;
    assert!(stopper.is_stopped());
    assert!(future::poll_once(stopper.into_future()).await.is_some());
}
