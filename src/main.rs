use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use blogs_api::{create_blog, get_blog, get_blogs, Blog, CreateBlogRequestPayload};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;
use tracing::{info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod blogs_api;

#[derive(OpenApi)]
#[openapi(
    paths(
        blogs_api::get_blogs,
        blogs_api::get_blog,
        blogs_api::create_blog
    ),
    components(
        schemas(Blog, CreateBlogRequestPayload)
    ),
    tags(
        (name = "blogs_api", description = "Blog management API")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // for logging
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // connecting to the database
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&url).await?;
    info!("Connected to Database.");

    let app = Router::new()
        .without_v07_checks()
        .route("/", get(index))
        .route("/blogs/{id}", get(get_blog))
        .route("/blogs", get(get_blogs).post(create_blog))
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(Extension(pool));

    // starting the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    info!("Application running at 127.0.0.1:8080");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn index() -> impl IntoResponse {
    Html("<h1>Ciao mondo!</h1>")
}
