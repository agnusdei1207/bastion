use axum::{
     routing::get, Extension,  Router
};
use routes::routes;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber;

mod handlers;
mod models;
mod routes;
mod cors;

use crate::cors::cors::create_cors;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let app: Router = Router::new()
        .route("/", get(root))
        .merge(routes())
        .layer(
            create_cors()
        );

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server is running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
 
    Ok(())
}
 
async fn root() -> &'static str {
    "Friede sei mit euch!"
}



