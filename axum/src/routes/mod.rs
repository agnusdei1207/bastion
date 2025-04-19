use axum::Router;

pub mod eve_json_log;
pub mod rule;

pub fn routes() -> Router {
    Router::new()
    .merge(eve_json_log::post_eve_json_log())
    .merge(rule::post_rule())
}