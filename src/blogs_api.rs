use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::PgPool;
use tracing::error;

use crate::{
    errors::ClientError,
    models::{Blog, BlogPostPayload, BlogUpdatePayload},
};

/// Create a new blog
///
/// Creates a new blog in the database, returns the created blog
#[utoipa::path(post, path = "/blogs", request_body = BlogPostPayload,
    responses(
            (status = 500, description = "Internal server error", body=ClientError),
            (status = 201, description = "Blog created successfully", body=Blog)
        )
)]
pub async fn create_blog(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<BlogPostPayload>,
) -> impl IntoResponse {
    let likes = body.likes.unwrap_or(0);

    match sqlx::query_as!(
        Blog,
        "INSERT INTO blogs (title, author, url, likes) VALUES ($1, $2, $3, $4) RETURNING id, title, author, url, likes",
        body.title,
        body.author,
        body.url,
        likes
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
pub async fn get_blogs(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
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
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Blog,
        "SELECT id, title, author, url, likes FROM blogs WHERE id = $1::int",
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
    request_body = BlogUpdatePayload,
    responses(
        (status = 500, description = "Internal server error", body=ClientError),
        (status = 200, description = "Blog updated successfully", body=Blog)
    )
)]
pub async fn update_blog(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    Json(body): Json<BlogUpdatePayload>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Blog,
        "UPDATE blogs SET title=$1, author=$2, url=$3, likes=$4::int WHERE id = $5 RETURNING id, title, author, url, likes",
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
    Extension(pool): Extension<PgPool>,
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

#[cfg(test)]
mod blog_api_test {
    use crate::test_helper::insert_test_values;
    use crate::{app, config::get_postgres_pool, test_helper::get_test_blogs};
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use rstest::*;
    use serde_json::{json, Value};
    use sqlx::{migrate, postgres::PgPoolOptions};
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;
    use tower::ServiceExt;

    // return value is an empty array if the database is empty
    // or returns 404 if single blog is requested
    #[rstest]
    #[case::get_blogs_1("/blogs/1", json!({"message": "Failed to retrieve blog"}), StatusCode::INTERNAL_SERVER_ERROR)]
    #[case::get_blogs("/blogs", json!([]), StatusCode::OK)]
    #[tokio::test(flavor = "multi_thread")]
    async fn get_blogs_empty_db(
        #[case] endpoint: &str,
        #[case] expected: Value,
        #[case] expected_status_code: StatusCode,
    ) {
        // create test container
        let container = Postgres::default().start().await.unwrap();
        let db_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            container.get_host_port_ipv4(5432).await.unwrap()
        );

        // connect to db pool
        let pool = get_postgres_pool(db_url.clone()).await;
        let db = PgPoolOptions::new().connect(&db_url).await.unwrap();
        migrate!("./migrations").run(&db).await.unwrap();

        // create server
        let app = app(pool.clone()).await;
        let server = TestServer::new(app).unwrap();

        // perform request
        let response = server.get(endpoint).await;
        let body: Value = response.json();

        response.assert_status(expected_status_code);
        assert_eq!(expected, body);

        // cleanup
        container.rm().await.unwrap();
    }

    // blogs are returned as json and are the correct amount
    #[rstest]
    #[case::get_single_blog("/blogs/1", json!(
        get_test_blogs()[0]
        ), StatusCode::OK)]
    #[case::get_multiple_blogs("/blogs", json!(get_test_blogs()), StatusCode::OK)]
    #[tokio::test]
    async fn get_blogs_correct_json_fields(
        #[case] endpoint: &str,
        #[case] expected: Value,
        #[case] expected_status_code: StatusCode,
    ) {
        // create test container
        let container = Postgres::default().start().await.unwrap();
        let db_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            container.get_host_port_ipv4(5432).await.unwrap()
        );

        // connect to db pool
        let pool = get_postgres_pool(db_url.clone()).await;
        let db = PgPoolOptions::new().connect(&db_url).await.unwrap();
        migrate!("./migrations").run(&db).await.unwrap();
        insert_test_values(&pool)
            .await
            .expect("Expected insert statement to work");

        // create server
        let app = app(pool.clone()).await;
        let server = TestServer::new(app).unwrap();

        // perform request
        let response = server.get(endpoint).await;
        let body: Value = response.json();

        response.assert_status(expected_status_code);
        assert_eq!(expected, body);

        // cleanup
        container.rm().await.unwrap();
    }
}
