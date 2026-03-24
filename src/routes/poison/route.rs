use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use reqwest::header;
use std::sync::Arc;
use tokio::sync::{Semaphore, TryAcquireError};

use super::fetch_poison::stream_poison;
use super::gzip;
use super::html_builder;
use crate::config::MiasmaConfig;

/// Miasma's poison serving trap.
pub async fn serve_poison(
    config: &'static MiasmaConfig,
    sem: Arc<Semaphore>,
    should_gzip: bool,
) -> impl IntoResponse {
    let permit = match sem.try_acquire_owned() {
        Ok(p) => p,
        Err(e) => match e {
            TryAcquireError::NoPermits => {
                return Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header(header::RETRY_AFTER, 5)
                    .body(Body::empty())
                    .unwrap();
            }
            TryAcquireError::Closed => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
    };

    let poison = match stream_poison(&config.poison_source).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error fetching from {}: {e}", config.poison_source);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let stream = html_builder::POSION_PAGE.build_html_stream(
        poison,
        config.link_count,
        &config.link_prefix,
        permit,
    );
    let body_stream = if should_gzip {
        Body::from_stream(gzip::gzip_stream(stream))
    } else {
        Body::from_stream(stream)
    };

    let mut builder = Response::builder().header(header::CONTENT_TYPE, "text/html");
    if should_gzip {
        builder = builder.header(header::CONTENT_ENCODING, "gzip");
    }
    builder.body(body_stream).unwrap_or_else(|e| {
        eprintln!("Failed to build poison route response: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}
