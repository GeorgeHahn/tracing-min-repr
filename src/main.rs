use std::time::Duration;
use tracing::{info, info_span, Instrument};
use tracing_subscriber::{
    filter::LevelFilter, fmt::format::FmtSpan, layer::SubscriberExt, Layer, Registry,
};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(inner_main());
    info!("Inner main finished, waiting for shutdown");
    rt.shutdown_timeout(Duration::from_secs(10));
    info!("Shutdown complete");
}

async fn inner_main() {
    let mut config = opentelemetry_sdk::trace::Config::default();
    config.sampler = Box::new(opentelemetry_sdk::trace::Sampler::AlwaysOn);
    config.id_generator = Box::new(opentelemetry_sdk::trace::RandomIdGenerator::default());

    opentelemetry_datadog::new_pipeline()
        .with_service_name("test-service")
        .with_api_version(opentelemetry_datadog::ApiVersion::Version05)
        .with_agent_endpoint("http://127.0.0.1:8126")
        .with_trace_config(config)
        .install_batch()
        .expect("installing APM pipeline failed");

    let opentelemetry_layer = tracing_opentelemetry::layer();

    let subscriber = Registry::default().with(opentelemetry_layer).with(
        tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::FULL)
            .with_filter(LevelFilter::TRACE),
    );

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    reproduce_span_outlives_parent_panic().await;
}

async fn reproduce_span_outlives_parent_panic() {
    let (top_level_exited, rx) = tokio::sync::oneshot::channel();
    let (child_complete, child_rx) = tokio::sync::oneshot::channel();
    top_level_task(rx, child_complete).await;
    top_level_exited.send(()).unwrap();
    child_rx.await.unwrap();
}

#[tracing::instrument(skip_all)]
async fn child_of_current() {
    info!("Child of current span");
}

#[tracing::instrument(skip_all)]
async fn top_level_task(
    wait_for_top_level_exit: tokio::sync::oneshot::Receiver<()>,
    child_complete: tokio::sync::oneshot::Sender<()>,
) {
    info!("Spawn task capturing parent span");

    let child_of_current = child_of_current().instrument(tracing::Span::current());

    let current_span = tracing::Span::current();
    tokio::spawn(async move {
        wait_for_top_level_exit.await.unwrap();

        child_of_current.await;

        // This doesn't panic
        info_span!(parent: &current_span, "child_operation1");

        // This panics
        info_span!(parent: current_span, "child_operation2");

        // Without this, about half of runs will exit before the child task runs. Of interest as a possible tokio bug.
        child_complete.send(()).unwrap();
    });
}
