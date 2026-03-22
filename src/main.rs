use std::sync::Arc;

use anyhow::Context;
use axum::routing::get;
use miasma::{MiasmaConfig, routes};
use tokio::sync::Semaphore;

// TODO: add async method to check version and report to user if a newer version can be installed

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = MiasmaConfig::parse();
    let MiasmaConfig {
        port,
        host,
        max_in_flight,
        poison_source,
        link_count,
        ..
    } = config.clone();

    let in_flight_sem = Arc::new(Semaphore::new(config.max_in_flight as usize));

    let app = axum::Router::new().fallback(get(move || async move {
        routes::serve_poison(&config, in_flight_sem).await
    }));

    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .with_context(|| format!("could not bind to {host}:{port}"))?;

    eprintln!("Listening on '{host}:{port}' with {max_in_flight} max in-flight requests...");
    eprintln!(
        "Serving poisoned training data from '{poison_source}' with {link_count} nested links per response..."
    );

    axum::serve(listener, app)
        .await
        .with_context(|| "server exited with an unexpected error")
}
