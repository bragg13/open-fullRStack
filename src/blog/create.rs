use crate::{
    errors::ClientError,
    models::{Blog, BlogPostPayload},
};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::PgPool;
use tracing::error;

/// Create a new blog
///
/// Creates a new blog in the database, returns the created blog
#[utoipa::path(post, path = "/blogs", request_body = BlogPostPayload,
    responses(
            (status = 500, description = "Internal server error", body=ClientError),
            (status = 400, description = "Bad request"),
            (status = 422, description = "Unprocessable entity" ),
            (status = 201, description = "Blog created successfully", body=Blog)
        )
)]
pub async fn create_blog(
    Extension(pool): Extension<PgPool>,
    Json(body): Json<BlogPostPayload>,
) -> impl IntoResponse {
    let likes = body.likes.unwrap_or(0);
    println!("{:?}", body);

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

// #[cfg(test)]
// mod test {
//     use crate::blog::test_helper::{get_test_blogs, insert_test_values};
//     use crate::models::BlogPostPayload;
//     use crate::{app, config::get_postgres_pool};
//     use axum::http::StatusCode;
//     use axum_test::TestServer;
//     use rstest::*;
//     use serde_json::{json, Value};
//     use sqlx::{migrate, postgres::PgPoolOptions};
//     use testcontainers::runners::AsyncRunner;
//     use testcontainers_modules::postgres::Postgres;
//     use tower::ServiceExt;

//     // a new blog can be correctly added via POST
//     // blog without likes field defaults to zero
//     // missing url or title in POST request should return status code 400
//     // TODO: a new blog cannot be added if a valid token is not provided
//     #[rstest]
//     #[case::works(
//         json!( {
//             "title": "React patterns".to_string(),
//             "author": "Michael Chan".to_string(),
//             "url": "https://reactpatterns.com/".to_string(),
//             "likes": Some(7),
//         }),
//         json!(
//         get_test_blogs()[0]
//         ), StatusCode::CREATED)]
//     #[case::no_likes(
//         json!( {
//             "title": "Type wars".to_string(),
//             "author": "Robert C. Martin".to_string(),
//             "url": "http://blog.cleancoder.com/uncle-bob/2017/03/03/TDD-Harms-Architecture.html".to_string(),
//         }),
//         json!(
//         get_test_blogs()[4]
//         ), StatusCode::CREATED)]
//     #[case::missing_url_and_title(
//         json!({
//             "author": "Robert C. Martin".to_string(),
//             "likes": Some(8)
//         }) ,
//         json!(
//         {"message": "Error"}
//         ), StatusCode::INTERNAL_SERVER_ERROR)]
//     #[tokio::test]
//     async fn post_blogs(
//         #[case] payload: Value,
//         #[case] expected: Value,
//         #[case] expected_status_code: StatusCode,
//     ) {
//         // create test container
//         let container = Postgres::default().start().await.unwrap();
//         let db_url = format!(
//             "postgresql://postgres:postgres@localhost:{}/postgres",
//             container.get_host_port_ipv4(5432).await.unwrap()
//         );

//         // connect to db pool
//         let pool = get_postgres_pool(db_url.clone()).await;
//         let db = PgPoolOptions::new().connect(&db_url).await.unwrap();
//         migrate!("./migrations").run(&db).await.unwrap();
//         insert_test_values(&pool)
//             .await
//             .expect("Expected insert statement to work");

//         // create server
//         let app = app(pool.clone()).await;
//         let server = TestServer::new(app).unwrap();

//         // perform request
//         let response = server.post("/blogs").json(&payload).await;
//         let body: Value = response.json();

//         response.assert_status(expected_status_code);
//         assert_eq!(expected, body);

//         // cleanup
//         container.rm().await.unwrap();
//     }
// }
