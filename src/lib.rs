mod config;
mod routes;

use std::sync::Arc;

use axum::{Router, routing::get};
pub use config::MiasmaConfig;
use tokio::sync::Semaphore;

/// Build a new `axum::Router` for miasma's routes.
pub fn new_miasma_router(config: &'static MiasmaConfig) -> Router {
    let in_flight_sem = Arc::new(Semaphore::new(config.max_in_flight as usize));

    Router::new().fallback(get(move || routes::serve_poison(config, in_flight_sem)))
}
