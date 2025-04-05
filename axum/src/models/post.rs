
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub user_id: Option<i32>,
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub body: String,
    pub user_id: Option<i32>
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePost {
    pub title: String,
    pub body: String,
    pub user_id: Option<i32>,
}