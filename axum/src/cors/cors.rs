use tower_http::cors::CorsLayer;
use axum;

pub fn create_cors() -> CorsLayer {
    CorsLayer::new()
    .allow_methods([
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::PUT,
        axum::http::Method::DELETE,
        axum::http::Method::PATCH,
        axum::http::Method::OPTIONS,
    ])
    .allow_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::header::AUTHORIZATION,
        axum::http::header::ACCEPT,
        axum::http::header::ORIGIN,
        axum::http::header::REFERER,
        axum::http::header::USER_AGENT,
    ])
    .allow_origin([
        "http://localhost:localhost:8080".parse().expect(
            "Invalid CORS origin"
        )
    ])
    .allow_credentials(true)
}