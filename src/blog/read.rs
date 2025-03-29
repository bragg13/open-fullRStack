use crate::{errors::ClientError, models::Blog};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::PgPool;
use tracing::error;

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
    path = "/blogs",
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

#[cfg(test)]
mod test {
    use crate::blog::test_helper::test_helper::{get_test_blogs, insert_test_values, spawn_app};
    use axum::http::StatusCode;
    use rstest::*;
    use serde_json::{json, Value};

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
        let app = spawn_app().await;
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}{}", app.address, endpoint))
            .send()
            .await
            .expect("Failed to execute request.");

        let status_code = response.status();
        let body: Value = response
            .json()
            .await
            .expect("Expected to parse response body");

        // Assert
        assert_eq!(status_code, expected_status_code);
        assert_eq!(body, expected);

        // cleanup
        app.container.rm().await.unwrap();
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
        let app = spawn_app().await;
        let client = reqwest::Client::new();
        insert_test_values(&app.db_pool)
            .await
            .expect("Expected insert statement to work");

        let response = client
            .get(&format!("{}{}", app.address, endpoint))
            .send()
            .await
            .expect("Failed to execute request.");

        let status_code = response.status();
        let body: Value = response
            .json()
            .await
            .expect("Expected to parse response body");

        // Assert
        assert_eq!(status_code, expected_status_code);
        assert_eq!(body, expected);

        // cleanup
        app.container.rm().await.unwrap();
    }
}
