mod favicon_response;

use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::str::FromStr;

use accept_header::Accept;
use axum::extract::{Path, Query};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use image::ImageFormat;
use lazy_static::lazy_static;
use mime::Mime;
use thiserror::Error;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use url::Url;

use crate::cli_args::ServerOptions;
use crate::get_favicon::{fetch_favicon, GetFaviconError};
use crate::DEFAULT_IMAGE_FORMAT;
use crate::DEFAULT_IMAGE_SIZE;

use self::favicon_response::FaviconResponse;

lazy_static! {
    static ref SUPPORTED_OUTPUT_MIME_TYPES: Vec<Mime> = {
        use ImageFormat::*;
        [
            Png, Jpeg, Gif, WebP, Pnm, Tiff, Tga, Dds, Bmp, Ico, Hdr, OpenExr, Farbfeld, Qoi,
        ]
        .into_iter()
        .map(|format| Mime::from_str(format.to_mime_type()).unwrap())
        .collect()
    };
}

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
    headers: HeaderMap,
) -> impl IntoResponse {
    // Determine requested size
    let size: Option<u32> = params.get("size").and_then(|s| s.parse().ok());

    // Determine requested format
    let format: Option<ImageFormat> = headers.get(axum::http::header::ACCEPT).and_then(|accept| {
        // Parse accept header, determine most desired content type
        let accept: Accept = accept.to_str().unwrap().parse().unwrap();
        let mime_type = accept
            .negotiate(&SUPPORTED_OUTPUT_MIME_TYPES)
            .unwrap()
            .to_string();
        ImageFormat::from_mime_type(mime_type)
    });

    // Parse the provided url
    let target_url = Url::parse(&target_url_input)
        .ok()
        .or_else(|| Url::parse(&format!("http://{}", target_url_input)).ok());

    // Get the favicon
    let favicon_res = match &target_url {
        Some(target_url) => fetch_favicon(target_url, size.unwrap_or(DEFAULT_IMAGE_SIZE)).await,
        None => Err(GetFaviconError::InvalidUrl),
    };

    // Construct a response
    FaviconResponse::from_fetch_result(
        favicon_res,
        target_url
            .and_then(|url| url.host_str().map(|s| s.to_owned()))
            .unwrap_or("?".to_owned()),
        size.unwrap_or(DEFAULT_IMAGE_SIZE),
        format.unwrap_or(DEFAULT_IMAGE_FORMAT),
    )
}
