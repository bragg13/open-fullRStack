use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct Blog {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub url: String,
    pub likes: i32,
}
#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct BlogPostPayload {
    pub title: String,
    pub author: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<i32>,
}
#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct BlogUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<i32>,
}
