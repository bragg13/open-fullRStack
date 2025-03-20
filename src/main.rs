use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use blogs_api::{create_blog, delete_blog, get_blog, get_blogs, update_blog};
use dotenvy::dotenv;
use models::{Blog, CreateBlogRequestPayload, UpdateBlogRequestPayload};
use sqlx::postgres::PgPoolOptions;
use std::{error::Error, net::SocketAddr};
use tracing::{error, info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod blogs_api;
mod errors;
mod models;

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
        schemas(Blog, CreateBlogRequestPayload, UpdateBlogRequestPayload)
    ),
    tags(
        (name = "blogs_api", description = "Blog management API")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // connecting to the database
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").map_err(|e| {
        error!("Failed to read DATABASE_URL: {} ", e);
        e
    })?;

    let pool = PgPoolOptions::new().connect(&url).await.map_err(|e| {
        error!("Failed to connect to the database: {} ", e);
        e
    })?;
    info!("Connected to Database.");

    let app = Router::new()
        .without_v07_checks()
        .route("/", get(index))
        .merge(routes_blogs())
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(Extension(pool));

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

fn routes_blogs() -> Router {
    Router::new()
        .route(
            "/blogs/{id}",
            get(get_blog).put(update_blog).delete(delete_blog),
        )
        .route("/blogs", get(get_blogs).post(create_blog))
}

async fn index() -> impl IntoResponse {
    Html("<h1>Ciao mondo!</h1>")
}
