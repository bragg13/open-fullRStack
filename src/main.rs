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

    let state = Arc::new(AppStateInner::new().await);
    let app = app().await;
    let app = app.with_state(state);

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

async fn app() -> Router<Arc<AppStateInner>> {
    Router::new()
        .without_v07_checks()
        .route("/", get(index))
        .route(
            "/blogs/{id}",
            get(get_blog).put(update_blog).delete(delete_blog),
        )
        .route("/blogs", get(get_blogs).post(create_blog))
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

async fn index() -> impl IntoResponse {
    Html("<h1>Ciao mondo!</h1>")
}

#[cfg(test)]
mod tests {
    use crate::{app, config::AppStateInner};
    use axum::http::StatusCode;
    use dotenvy::dotenv;
    use httpc_test::{self, Client};
    use rstest::*;
    use std::{sync::Arc, time::Duration};
    use tokio::{net::TcpListener, sync::OnceCell, time::sleep};
    static TEST_CLIENT: OnceCell<Arc<Client>> = OnceCell::const_new();

    async fn get_test_client() -> Arc<Client> {
        async fn setup() -> Arc<Client> {
            // Bind the server to a test port.
            let listener = TcpListener::bind("127.0.0.1:8081")
                .await
                .expect("Failed to bind to test port");

            // Create the state and app.
            dotenv().ok();
            let state = Arc::new(AppStateInner::new().await);
            let app = app().await.with_state(state);

            // Spawn the server in the background.
            tokio::spawn(async move {
                axum::serve(listener, app)
                    .await
                    .expect("Server failed during test");
            });

            // Give the server a moment to start up.
            sleep(Duration::from_millis(100)).await;

            // Create and return the test client.
            Arc::new(
                httpc_test::new_client("http://127.0.0.1:8081")
                    .expect("Failed to create test client"),
            )
        }

        TEST_CLIENT.get_or_init(setup).await.clone()
    }

    #[rstest]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_blogs() {
        let client = get_test_client().await;
        let response = client
            .do_get("/blogs")
            .await
            .expect("GET /blogs should work fine");

        // response.print().await.ok();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
