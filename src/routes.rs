mod html_builder;
mod poison;

use std::sync::Arc;

use axum::{http::StatusCode, response::Html};
use tokio::sync::{Semaphore, TryAcquireError};

use crate::MiasmaConfig;

// TODO: stream resonse rather than creating HTML struct
// TODO: compress response to save on bandwith costs

/// Miasma's poison serving trap.
pub async fn serve_poison(
    config: &MiasmaConfig,
    sem: Arc<Semaphore>,
) -> (StatusCode, Html<String>) {
    let _permit = match sem.try_acquire() {
        Ok(p) => p,
        Err(e) => match e {
            // TODO: include Retry-After header
            TryAcquireError::NoPermits => {
                return (StatusCode::TOO_MANY_REQUESTS, Html(String::new()));
            }
            TryAcquireError::Closed => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new()));
            }
        },
    };

    let poison = match poison::fetch_poison(&config.poison_source).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error fetching from poison source: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new()));
        }
    };

    let page = html_builder::POSION_PAGE
        .build_html_str(&poison, config.link_count, &config.link_prefix)
        .await;

    (StatusCode::OK, Html(page))
}
