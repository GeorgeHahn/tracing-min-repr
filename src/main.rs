use std::time::Duration;
use tracing::info;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    tracing_subscriber::fmt::init();

    rt.block_on(reproduce_span_outlives_parent_panic());
    info!("Inner main finished, waiting for shutdown");
    rt.shutdown_timeout(Duration::from_secs(10));
    info!("Shutdown complete");
}

async fn reproduce_span_outlives_parent_panic() {
    let (top_level_exited, rx) = tokio::sync::oneshot::channel();
    let (child_complete, child_rx) = tokio::sync::oneshot::channel();
    top_level_task(rx, child_complete).await;
    top_level_exited.send(()).unwrap();

    // With this line commented out, about three quarters of runs do not finish the spawned task below.
    // child_rx.await.unwrap();
}

#[tracing::instrument(skip_all)]
async fn top_level_task(
    wait_for_top_level_exit: tokio::sync::oneshot::Receiver<()>,
    child_complete: tokio::sync::oneshot::Sender<()>,
) {
    info!("Spawn task capturing parent span");

    tokio::spawn(async move {
        wait_for_top_level_exit.await.unwrap();

        // Panics if it runs: receiver has been dropped
        child_complete.send(()).unwrap();
    });
}
