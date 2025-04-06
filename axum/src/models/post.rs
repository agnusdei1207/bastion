
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub user_id: Option<i32>,
    pub title: String,
    pub body: String,
}
