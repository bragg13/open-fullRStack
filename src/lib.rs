use axum::{
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};

use models::{Blog, BlogUpdatePayload};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
pub mod blog;
mod configuration;
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

pub async fn run(listener: TcpListener, db_pool: PgPool) -> Result<(), std::io::Error> {
    let app = app(db_pool).await;
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

async fn health_check() -> Response {
    info!("Health check!");
    Response::default()
}
