use axum::Router;

pub mod eve_json_log;

pub fn routes() -> Router {
    Router::new()
    .merge(eve_json_log::post_eve_json_log())
}