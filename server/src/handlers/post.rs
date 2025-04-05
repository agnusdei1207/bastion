use axum::{
    extract::Path, http::StatusCode, Extension, Json, 
};
use crate::models::post::{CreatePost, Post, UpdatePost};
use reqwest;

pub async fn create_post(
    Json(new_post): Json<CreatePost>
) -> Result<Json<Post>, StatusCode>{
    let client = reqwest::Client::new();
    // Define your external API URL
    let external_api_url = "https://your-external-api-endpoint.com/path";
    
    // Forward the received body to the external API
    let response = client
        .post(external_api_url)
        .json(&new_post)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Check if the request to the external API was successful
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    
    // Parse the response from the external API
    let post: Post = response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(post))
}