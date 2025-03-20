use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use blogs_api::{create_blog, delete_blog, get_blog, get_blogs, update_blog};
use config::DbConfig;
use dotenvy::dotenv;
use models::{Blog, CreateBlogRequestPayload, UpdateBlogRequestPayload};
use sqlx::{postgres::PgPoolOptions, Error, Pool, Postgres};
use std::net::SocketAddr;
use tracing::{error, info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod blogs_api;
mod config;
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").map_err(|e| {
        error!("Failed to read DATABASE_URL: {} ", e);
        e
    })?;
    let db_config = DbConfig { url: db_url };
    let pool = connect_to_postgres(db_config).await?;
    let app = app(pool);

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

async fn connect_to_postgres(config: DbConfig) -> Result<Pool<Postgres>, Error> {
    match PgPoolOptions::new().connect(&config.url).await {
        Ok(pool) => {
            info!("Connected to Database.");
            Ok(pool)
        }
        Err(e) => Err(e),
    }
}

fn app(pool: Pool<Postgres>) -> Router {
    Router::new()
        .without_v07_checks()
        .route("/", get(index))
        .merge(router_blogs())
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(Extension(pool))
}

fn router_blogs() -> Router {
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn index_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
