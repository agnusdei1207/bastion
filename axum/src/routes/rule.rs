use axum::{
    routing::post,
    Router,
};

use crate::handlers::rule::add_rule;

pub fn post_rule() -> Router {
    Router::new()
    .route("/rule", post(add_rule))
}