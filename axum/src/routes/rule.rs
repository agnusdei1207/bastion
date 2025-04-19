use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::rule::{add_rule, delete_rule, get_rule_by_id, list_rules};

pub fn post_rule() -> Router {
    Router::new()
    .route("/rules", get(list_rules))
    .route("/rules/:id" , get(get_rule_by_id))
    .route("/rules", post(add_rule))
    .route("/rules/:id", delete(delete_rule))
}