use axum::Router;

pub mod post;

pub fn routes() -> Router {
    Router::new()
    .merge(post::post_route())
}