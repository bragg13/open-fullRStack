use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use blogs_api::{create_blog, delete_blog, get_blog, get_blogs, update_blog};
use config::AppStateInner;
use dotenvy::dotenv;
use models::{Blog, UpdateBlogRequestPayload};
use std::{net::SocketAddr, sync::Arc};
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
        schemas(Blog,  UpdateBlogRequestPayload)
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

    let app = app().await;

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

async fn app() -> Router {
    let state = Arc::new(AppStateInner::new().await);

    Router::new()
        .without_v07_checks()
        .route("/", get(index))
        .route(
            "/blogs/{id}",
            get(get_blog).put(update_blog).delete(delete_blog),
        )
        .route("/blogs", get(get_blogs).post(create_blog))
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

async fn index() -> impl IntoResponse {
    Html("<h1>Ciao mondo!</h1>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use rstest::*;

    #[derive(Clone)]
    struct AppState {}

    macro_rules! block_on {
        ($async_expr:expr) => {{
            tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::current();
                handle.block_on($async_expr)
            })
        }};
    }

    // #[fixture]
    // #[once]
    // fn test_router() -> Router {
    //     dotenv().ok();
    //     let db_url = std::env::var("DATABASE_URL_TEST").unwrap();
    //     let db_config = DbConfig { url: db_url };
    //     let pool = block_on!(async { connect_to_postgres(db_config).await.unwrap() });
    //     app(pool)
    // }
    #[rstest]
    #[tokio::test]
    async fn test_get_blog() {
        // let router = test_router().with_state(());
        // let response = router
        //     .oneshot(
        //         Request::builder()
        //             .uri("/blogs/1")
        //             .body(axum::body::Body::empty())
        //             .unwrap(),
        //     )
        //     .await
        //     .unwrap();
        // assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    // async fn not_found() {
    //     assert_eq!(response.status(), StatusCode::NOT_FOUND);
    //     let body = response.into_body().collect().await.unwrap().to_bytes();
    //     assert!(body.is_empty());
    // }
}
