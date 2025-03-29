use axum::{
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};

use config::{get_db_url, get_postgres_pool};
use models::{Blog, BlogUpdatePayload};
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::{error, info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
pub mod blog;
mod config;
mod errors;
mod models;

#[derive(OpenApi)]
#[openapi(
    paths(
        blog::create::create_blog,
        blog::read::get_blog,
        blog::read::get_blogs,
        blog::update::update_blog,
        blog::delete::delete_blog,
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
    let blog_router = blog::routes();
    Router::new()
        .without_v07_checks()
        .route("/", get(index))
        .route("/health_check", get(health_check))
        .merge(blog_router)
        .merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
        .layer(Extension(pool))
}

async fn index() -> impl IntoResponse {
    Html("<h1>Ciao mondo!</h1>")
}

#[axum::debug_handler]
async fn health_check() -> Response {
    info!("Health check!");
    Response::default()
}
