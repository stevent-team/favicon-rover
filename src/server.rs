use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::str::FromStr;

use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use thiserror::Error;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use url::Url;

use crate::cli_args::ServerOptions;
use crate::fallback::generate_fallback;
use crate::favicon::Favicon;
use crate::get_favicon::{get_favicon, GetFaviconError, DEFAULT_IMAGE_SIZE};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    InvalidHost(#[from] AddrParseError),
}

pub async fn start_server(options: ServerOptions) -> Result<(), ServerError> {
    // Init tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    // Define axum app
    let app = Router::new()
        .route("/:path", get(get_favicon_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    // Parse address
    let addr = IpAddr::from_str(&options.host)?;
    let addr = SocketAddr::new(addr, options.port);

    // Start server
    tracing::info!("Starting favicon rover on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler")
        })
        .await
        .unwrap();

    Ok(())
}

async fn get_favicon_handler(
    Path(target_url_input): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Determine requested size
    let size: Option<u32> = params.get("size").and_then(|s| s.parse().ok());

    // Parse the provided url
    let target_url = Url::parse(&target_url_input)
        .ok()
        .or_else(|| Url::parse(&format!("http://{}", target_url_input)).ok());

    // Get the favicon and send it
    match target_url {
        Some(target_url) => get_favicon(&target_url, size).await,
        None => Favicon::Fallback(
            generate_fallback(target_url_input, size.unwrap_or(DEFAULT_IMAGE_SIZE)),
            GetFaviconError::InvalidUrl,
        ),
    }
}
