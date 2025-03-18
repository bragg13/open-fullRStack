use axum::{extract::Path, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::{error, info};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Blog {
    id: i32,
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
    likes: Option<i32>,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateBlogRequestPayload {
    title: Option<String>,
    author: Option<String>,
    url: Option<String>,
    likes: Option<i32>,
}

/// Create a new blog
///
/// Creates a new blog in the database, returns the created blog
#[utoipa::path(post, path = "/blogs", request_body = CreateBlogRequestPayload,
    responses(
            (status = 200, description = "Blog created successfully", body = Blog),
            (status = 500, description = "Internal server error")
        )
)]
pub async fn create_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(body): Json<CreateBlogRequestPayload>,
) -> Result<Json<Blog>, StatusCode> {
    let blog = sqlx::query_as!(
        Blog,
        "INSERT INTO blogs (title, author, url, likes) VALUES ($1, $2, $3, $4) RETURNING id, title, author, url, likes",
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

    info!("Created Blog with id={}", blog.id);

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
    info!("Fetched {} blogs from database", blogs.len());

    Ok(Json(blogs))
}

/// Get one blog
///
/// Returns a blog from the database given the id
#[utoipa::path(
    get,
    path = "/blogs/{id}",
    responses(
        (status = 200, description = "Blog returned successfully", body = Blog),
        (status = 404, description = "Blog not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<Blog>, StatusCode> {
    let blog = sqlx::query_as!(
        Blog,
        "SELECT id, title, author, url, likes FROM blogs WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; // TODO better error handling
    info!(
        "Fetched one blog with id={} and author={} from database",
        blog.id, blog.author
    );

    Ok(Json(blog))
}

/// Update one blog
///
/// Returns a blog from the database given the id
#[utoipa::path(
    put,
    path = "/blogs/{id}",
    request_body = UpdateBlogRequestPayload,
    responses(
        (status = 200, description = "Blog updated successfully", body = Blog),
        (status = 404, description = "Blog not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_blog(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateBlogRequestPayload>,
) -> Result<Json<Blog>, StatusCode> {
    let blog = sqlx::query_as!(
        Blog,
        "UPDATE blogs SET title=$1, author=$2, url=$3, likes=$4 WHERE id = $5 RETURNING id, title, author, url, likes",
        body.title,
        body.author,
        body.url,
        body.likes,
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; // TODO better error handling
    info!(
        "Fetched one blog with id={} and author={} from database",
        blog.id, blog.author
    );

    Ok(Json(blog))
}
