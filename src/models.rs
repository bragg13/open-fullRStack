use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Blog {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub url: String,
    pub likes: i32,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateBlogRequestPayload {
    pub title: String,
    pub author: String,
    pub url: String,
    pub likes: Option<i32>,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateBlogRequestPayload {
    pub title: Option<String>,
    pub author: Option<String>,
    pub url: Option<String>,
    pub likes: Option<i32>,
}
