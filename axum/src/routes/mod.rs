use axum::Router;

pub mod eve_json_log;
pub mod rule;
pub mod suricata;

pub fn routes() -> Router {
    Router::new()
    .merge(eve_json_log::router_eve_json_log())
    .merge(rule::router_rule())
    .merge(suricata::router_suricata())
}