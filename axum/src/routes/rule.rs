use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::rule::{create_rule, delete_rule, get_rule, get_rules};

pub fn router_rule() -> Router {
    Router::new()
        .nest(
            "/rule", 
            Router::new()
                .route("/", get(get_rules))
                .route("/{id}", get(get_rule))
                .route("/", post(create_rule))
                .route("/{id}", delete(delete_rule))
        )
}