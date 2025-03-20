use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::{Pool, Postgres};
use tracing::error;

use crate::{
    errors::ClientError,
    models::{Blog, CreateBlogRequestPayload, UpdateBlogRequestPayload},
};

/// Create a new blog
///
/// Creates a new blog in the database, returns the created blog
#[utoipa::path(post, path = "/blogs", request_body = CreateBlogRequestPayload,
    responses(
            (status = 500, description = "Internal server error", body=ClientError),
            (status = 201, description = "Blog created successfully", body=Blog)
        )
)]
pub async fn create_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(body): Json<CreateBlogRequestPayload>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Blog,
        "INSERT INTO blogs (title, author, url, likes) VALUES ($1, $2, $3, $4) RETURNING id, title, author, url, likes",
        body.title,
        body.author,
        body.url,
        body.likes
    )
    .fetch_one(&pool)
    .await
    {
        Ok(blog) =>
            (StatusCode::CREATED, Json(blog) ).into_response()
        ,
        Err(e) => {
            error!("Failed to create blog: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ClientError {message: "Failed to create blog".to_string()})).into_response()
        }
    }
}

/// Get all blogs
///
/// Returns a list of all blogs from the database
#[utoipa::path(
    get,
    path = "/blogs",
    responses(
        (status = 500, description = "Internal server error", body=ClientError),
        (status = 200, description = "Blogs retrieved successfully", body=[Blog])
    )
)]
pub async fn get_blogs(Extension(pool): Extension<Pool<Postgres>>) -> impl IntoResponse {
    match sqlx::query_as!(Blog, "SELECT id, title, author, url, likes FROM blogs")
        .fetch_all(&pool)
        .await
    {
        Ok(blogs) => (StatusCode::OK, Json(blogs)).into_response(),
        Err(e) => {
            error!("Failed to retrieve blogs: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ClientError {
                    message: "Failed to retrieve blogs".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get one blog
///
/// Returns a blog from the database given the id
#[utoipa::path(
    get,
    path = "/blogs/{id}",
    responses(
        (status = 500, description = "Internal server error", body=ClientError),
        (status = 200, description = "Blog retrieved successfully", body=Blog)
    )
)]
pub async fn get_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Blog,
        "SELECT id, title, author, url, likes FROM blogs WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(blog) => (StatusCode::OK, Json(blog)).into_response(),
        Err(e) => {
            error!("Failed to retrieve blog with id={}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ClientError {
                    message: "Failed to retrieve blog".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Update one blog
///
/// Returns a blog from the database given the id
#[utoipa::path(
    put,
    path = "/blogs/{id}",
    request_body = UpdateBlogRequestPayload,
    responses(
        (status = 500, description = "Internal server error", body=ClientError),
        (status = 200, description = "Blog updated successfully", body=Blog)
    )
)]
pub async fn update_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateBlogRequestPayload>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Blog,
        "UPDATE blogs SET title=$1, author=$2, url=$3, likes=$4 WHERE id = $5 RETURNING id, title, author, url, likes",
        body.title,
        body.author,
        body.url,
        body.likes,
        id
    )
    .fetch_one(&pool)
    .await {
        Ok(blog) => (StatusCode::OK, Json(blog)).into_response(),
        Err(e) => {
            error!("Failed to update blog with id={}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ClientError {
                    message: "Failed to update blog".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Delete a blog
///
/// Deletes a blog from the database given the id
#[utoipa::path(
    delete,
    path = "/blogs/{id}",
    responses(
        (status = 500, description = "Failed to delete blog"),
        (status = 200, description = "Blog deleted successfully")
    )
)]
pub async fn delete_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match sqlx::query!("DELETE FROM blogs WHERE id = $1", id)
        .execute(&pool)
        .await
    {
        Ok(_) => (StatusCode::OK, Json({})).into_response(),
        Err(e) => {
            error!("Failed to delete blog with id={}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ClientError {
                    message: "Failed to delete blog".to_string(),
                }),
            )
                .into_response()
        }
    }
}
