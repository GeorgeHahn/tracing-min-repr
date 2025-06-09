use std::time::Duration;
use tracing::info;

fn main() {
    tracing_subscriber::fmt().with_env_filter("trace").init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(top_level_task());
    info!("Top level task finished, waiting for shutdown");
    rt.shutdown_timeout(Duration::from_secs(10));
    info!("Shutdown complete");
}

async fn top_level_task() {
    let (top_level_exited, rx) = tokio::sync::oneshot::channel();
    let (child_complete, child_rx) = tokio::sync::oneshot::channel();
    inner_task(rx, child_complete).await;
    top_level_exited.send(()).unwrap();

    // With this line commented out, about three quarters of runs do not finish the spawned task below.
    // child_rx.await.unwrap();
}

#[tracing::instrument(skip_all)]
async fn inner_task(
    wait_for_top_level_exit: tokio::sync::oneshot::Receiver<()>,
    child_complete: tokio::sync::oneshot::Sender<()>,
) {
    info!("Spawn from inner task");

    tokio::spawn(async move {
        // Successful runs don't show this message:
        info!("Child task waiting");
        wait_for_top_level_exit.await.unwrap();

        // Panics if it runs: receiver has been dropped
        child_complete.send(()).unwrap();
    });
}
