use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::suricata::{get_suricata_rule_statistics, get_suricata_status, reload_suricata_rules};


pub fn router_suricata() -> Router {
    Router::new()
    .route("/suricata/status", get(get_suricata_status))
    .route("/suricata/statistics", get(get_suricata_rule_statistics))
    .route("/suricata/rules/reload", post(reload_suricata_rules))

}