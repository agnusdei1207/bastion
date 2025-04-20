use axum::{
    routing::post,
    Router,
};

use crate::handlers::eve_json_log::send_eve_json_log;

pub fn router_eve_json_log() -> Router {
    Router::new()
    .route("/eve_json_log", post(send_eve_json_log))
}