use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::suricata::{
    get_interface_statistics, 
    get_suricata_rule_statistics, 
    get_suricata_status, 
    reload_suricata_rules
};

pub fn router_suricata() -> Router {
    Router::new()
        .nest(
            "/suricata", 
            Router::new()
                .route("/status", get(get_suricata_status))
                .route("/statistics", get(get_suricata_rule_statistics))
                .route("/interface", get(get_interface_statistics))
                .route("/rules/reload", post(reload_suricata_rules))
        )
}