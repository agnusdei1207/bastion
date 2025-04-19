use axum::{
     routing::get, Error, Extension, Router
};
use routes::routes;
use tracing::{info, Level};
use tracing_subscriber;

mod handlers;
mod models;
mod routes;
mod cors;

use crate::cors::cors::create_cors;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let app: Router = Router::new()
        .route("/", get(root))
        .merge(routes())
        .layer(
            create_cors()
        );

    let listener: tokio::net::TcpListener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await{
        Ok(listener)=> listener,
        Err(e)=>{
            eprint!("Failed to bind to port 3000: {}", e);
            return Err(Error::new(e.to_string()));
        }
    };

    info!("Server is running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
 
    Ok(())
}
 
async fn root() -> &'static str {
    "Friede sei mit euch!"
}



