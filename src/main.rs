use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use blogs_api::{create_blog, delete_blog, get_blog, get_blogs, update_blog};
use config::{get_db_url, get_postgres_pool};
use models::{Blog, BlogUpdatePayload};
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tracing::{error, info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod blogs_api;
mod config;
mod errors;
mod models;
#[cfg(test)]
mod test_helper;
use tower::ServiceExt;

#[derive(OpenApi)]
#[openapi(
    paths(
        blogs_api::get_blogs,
        blogs_api::get_blog,
        blogs_api::update_blog,
        blogs_api::delete_blog,
        blogs_api::create_blog
    ),
    components(
        schemas(Blog,  BlogUpdatePayload)
    ),
    tags(
        (name = "blogs_api", description = "Blog management API")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let db_url = get_db_url(false);
    let pool = get_postgres_pool(db_url).await;
    let app = app(pool).await;

    // starting the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        error!("Failed to bind to address: {} ", e);
        e
    })?;
    info!("Application running at {}", &addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn app(pool: PgPool) -> Router {
    Router::new()
        .without_v07_checks()
        .route("/", get(index))
        .route(
            "/blogs/{id}",
            get(get_blog).put(update_blog).delete(delete_blog),
        )
        .route("/blogs", get(get_blogs).post(create_blog))
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(Extension(pool))
}

async fn index() -> impl IntoResponse {
    Html("<h1>Ciao mondo!</h1>")
}

// test('blog without likes field defaults to zero', async () => {
// test('a new blog can be correctly added via POST', async () => {
// test('a new blog cannot be added if a valid token is not provided', async () => {
// test('missing url or title in POST request should return status code 400', async () => {
// #[rstest]
// #[tokio::test]
// async fn test_post_blogs() {
//     let client = get_test_client().await;

//     let blog = json!( {
//         "author": "andrea",
//         "title": "blog1",
//         "url": "http://blog1.com",
//         "likes": 10,
//     });
//     empty_blogs_table(&server.pool).await.ok();
//     let blogs = insert_test_values(&server.pool)
//         .await
//         .expect("Expected insert statement to work");
//     println!("{:?}", blogs);
//     let response = client
//         .do_post("/blogs", blog)
//         .await
//         .expect("Expected POST /blogs to work");
//     assert_eq!(2, 2)
//     // POSTing a blog without likes parameter defaults it to zero
//     //
//     // POSTing without any other parameter returns error

//     // response.print().await.ok();
//     // assert_eq!(response.status(), StatusCode::OK);
// }

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
