use axum::{
    routing::{post},
    Router,
};

use crate::handlers::post::{create_post};

pub fn post_route() -> Router {
    Router::new()
    .route("/post", post(create_post))
}