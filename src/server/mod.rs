//! HTTP Server for fetching favicons by URL

mod fallback;
mod favicon_response;

use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::OnceLock;

use accept_header::Accept;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method};
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use image::ImageFormat;
use lazy_static::lazy_static;
use mime::Mime;
use regex::Regex;
use reqwest::Client;
use thiserror::Error;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;

use crate::cli_args::ServerOptions;
use crate::favicon_image::fetch::FetchFaviconError;
use crate::favicon_image::FaviconImage;
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

enum CorsOrigin {
    Regex(Regex),
    String(String),
}

static CORS_ORIGINS: OnceLock<Vec<CorsOrigin>> = OnceLock::new();
fn cors_origins(origins: &[String]) -> &'static Vec<CorsOrigin> {
    CORS_ORIGINS.get_or_init(|| {
        origins
            .iter()
            .map(|o| {
                if o.starts_with('/') && o.ends_with('/') {
                    // Remove the first and last slash
                    CorsOrigin::Regex(Regex::new(o.split_at(1).1.split_at(o.len() - 2).0).unwrap())
                } else {
                    CorsOrigin::String(o.to_owned())
                }
            })
            .collect()
    })
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    InvalidHost(#[from] AddrParseError),
}

#[derive(Debug, Clone)]
struct ServerState {
    client: Client,
}

pub async fn start_server(options: ServerOptions) -> Result<(), ServerError> {
    // Init tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .compact(),
        )
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // Cors
    let mut cors = CorsLayer::new().allow_headers(Any).allow_methods([
        Method::GET,
        Method::OPTIONS,
        Method::HEAD,
    ]);

    if options.origin.len() == 1 && options.origin[0] == "*" {
        cors = cors.allow_origin(Any)
    } else if options.origin.len() > 1
        && options
            .origin
            .iter()
            .all(|o| !o.starts_with('/') && !o.ends_with('/'))
    {
        cors = cors.allow_origin(
            options
                .origin
                .iter()
                .map(|o| o.parse().unwrap())
                .collect::<Vec<_>>(),
        )
    } else {
        cors = cors.allow_origin(AllowOrigin::predicate(move |origin, _| {
            cors_origins(&options.origin).iter().any(|o| match o {
                CorsOrigin::Regex(re) => re.is_match(origin.to_str().unwrap()),
                CorsOrigin::String(o) => o == origin.to_str().unwrap(),
            })
        }))
    }

    // Create axum state
    let state = ServerState {
        client: Client::new(),
    };

    // Define axum app
    let app = Router::new()
        .route("/", get(|| async { "Favicon Rover" }))
        .route("/:path", get(get_favicon_handler))
        .with_state(state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
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
                .expect("Failed to install Ctrl+C handler");
        })
        .await
        .unwrap();

    Ok(())
}

async fn get_favicon_handler(
    State(state): State<ServerState>,
    Path(target_url_input): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    tracing::info!("Get favicon for {target_url_input:?}");

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
        Some(target_url) => {
            FaviconImage::fetch_for_url(
                &state.client,
                target_url,
                size.unwrap_or(DEFAULT_IMAGE_SIZE),
            )
            .await
        }
        None => Err(FetchFaviconError::InvalidUrl),
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
