mod config;
mod routes;

pub use config::MiasmaConfig;

use axum::{Router, body::Body, http::Request, routing::get};
use reqwest::header::ACCEPT_ENCODING;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Build a new `axum::Router` for miasma's routes.
pub fn new_miasma_router(config: &'static MiasmaConfig) -> Router {
    let in_flight_sem = Arc::new(Semaphore::new(config.max_in_flight as usize));

    Router::new().fallback(get(move |req: Request<Body>| {
        let client_accepts_gzip = req
            .headers()
            .get(ACCEPT_ENCODING)
            .map(|acc| {
                acc.to_str()
                    .unwrap_or("")
                    .split(',')
                    // Don't you dare allocate anything !
                    .any(|tok| tok.trim().eq_ignore_ascii_case("gzip"))
            })
            .unwrap_or(false);

        routes::serve_poison(config, in_flight_sem, client_accepts_gzip)
    }))
}
