use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::handlers::post::{create_post, delete_post, get_post, get_posts, update_post};

pub fn post_route() -> Router {
    Router::new()
    .route("/posts", get(get_posts).post(create_post))
    .route("/posts/{id}", get(get_post).put(update_post).delete(delete_post))
}