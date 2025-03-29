use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::PgPool;
use tracing::error;

use crate::{
    errors::ClientError,
    models::{Blog, BlogUpdatePayload},
};

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

//     #[rstest]
//     #[tokio::test]
// test('a blog can be correctly updated via PUT', async () =>
//     async fn test_update_blogs() {
//         let client = get_test_client().await;
//         let server = get_test_server().await;

//         // update blog with some params
//         // update blog with no params doesnt change anyuthing
//         // error when id is wrong
//     }
// }
