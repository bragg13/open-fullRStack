use axum::{http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::{error, info};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Blog {
    id: i64,
    title: String,
    author: String,
    url: String,
    likes: i32,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateBlogRequestPayload {
    title: String,
    author: String,
    url: String,
    likes: i32,
}

/// Create a new blog
///
/// Craetes a new blog in the database
#[utoipa::path(post, path = "/blogs", request_body = CreateBlogRequestPayload,
    responses(
            (status = 200, description = "Blog created successfully", body = i32),
            (status = 500, description = "Internal server error")
        )
)]
pub async fn create_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(body): Json<CreateBlogRequestPayload>,
) -> Result<Json<i32>, StatusCode> {
    let blog = sqlx::query_scalar!(
        "INSERT INTO blogs (title, author, url, likes) VALUES ($1, $2, $3, $4) RETURNING id",
        body.title,
        body.author,
        body.url,
        body.likes
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Successfully created `Blog` with id={}", blog);

    Ok(Json(blog))
}

/// Get all blogs
///
/// Returns a list of all blogs from the database
#[utoipa::path(
    get,
    path = "/blogs",
    responses(
        (status = 200, description = "List of blogs returned successfully", body = Vec<Blog>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_blogs(
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<Json<Vec<Blog>>, StatusCode> {
    let blogs = sqlx::query_as!(Blog, "SELECT id, title, author, url, likes FROM blogs")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; // TODO better error handling
    info!("Successfully fetched `Blog` data from database");

    Ok(Json(blogs))
}
