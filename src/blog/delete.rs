use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::PgPool;
use tracing::error;

use crate::errors::ClientError;

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

// #[rstest]
// #[tokio::test]
// test('an existing blog can be correctly deleted via DELETE', async () => {
// async fn test_delete_blogs() {
//     let client = get_test_client().await;
//     let server = get_test_server().await;

//     // delete blog with right id
//     //
//     // error when id is wrong
// }
