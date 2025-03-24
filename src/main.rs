use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use blogs_api::{create_blog, delete_blog, get_blog, get_blogs, update_blog};
use config::{get_db_url, AppStateInner};
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

    let db_url = get_db_url(false);
    let state = Arc::new(AppStateInner::new(db_url).await);
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
    use crate::{
        app,
        config::{get_db_url, AppStateInner},
    };
    use axum::http::StatusCode;
    use httpc_test::{self, Client};
    use rstest::*;
    use sqlx::PgPool;
    use std::{sync::Arc, time::Duration};
    use tokio::{net::TcpListener, sync::OnceCell, time::sleep};
    static TEST_CLIENT: OnceCell<Arc<Client>> = OnceCell::const_new();

    async fn setup_test_db(pool: &PgPool) -> Result<(), sqlx::Error> {
        // drop the test table
        sqlx::query("DROP TABLE IF EXISTS test_blogs")
            .execute(pool)
            .await?;

        // create a new one
        sqlx::query(
            r#"
            CREATE TABLE test_blogs (
            id SERIAL PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            url TEXT NOT NULL,
            likes INT NULL DEFAULT 0,
            created_at TIMESTAMP DEFAULT NOW ()
        );
        "#,
        )
        .execute(pool)
        .await?;

        // insert test entries
        let titles = vec![
            "React patterns".to_string(),
            "Go To Statement Considered Harmful".to_string(),
            "Canonical string reduction".to_string(),
            "First class tests".to_string(),
            "TDD harms architecture".to_string(),
            "Type wars".to_string(),
        ];
        let authors = vec![
            "Michael Chan".to_string(),
            "Edsger W. Dijkstra".to_string(),
            "Edsger W. Dijkstra".to_string(),
            "Robert C. Martin".to_string(),
            "Robert C. Martin".to_string(),
            "Robert C. Martin".to_string(),
        ];
        let urls = vec![
            "https://reactpatterns.com/".to_string(),
            "http://www.u.arizona.edu/~rubinson/copyright_violations/Go_To_Considered_Harmful.html"
                .to_string(),
            "http://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD808.html".to_string(),
            "http://blog.cleancoder.com/uncle-bob/2017/05/05/TestDefinitions.htmll".to_string(),
            "http://blog.cleancoder.com/uncle-bob/2017/03/03/TDD-Harms-Architecture.html"
                .to_string(),
            "http://blog.cleancoder.com/uncle-bob/2016/05/01/TypeWars.html".to_string(),
        ];
        let likes = vec![Some(7), Some(5), Some(12), Some(10), Some(0), Some(2)];

        sqlx::query(
            r#" INSERT INTO test_blogs (title, author, url, likes) SELECT * FROM UNNEST($1, $2, $3, $4)"#)
            .bind(titles)
            .bind(authors)
            .bind(urls)
            .bind(likes)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn setup() -> Arc<Client> {
        // Bind the server to a test port.
        let listener = TcpListener::bind("127.0.0.1:8081")
            .await
            .expect("Failed to bind to test port");

        // Create the state and app.
        let db_url = get_db_url(true);
        let state = Arc::new(AppStateInner::new(db_url).await);
        match setup_test_db(&state.pool).await {
            Ok(_) => println!("DB initialised."),
            Err(e) => println!("Error: {e}"),
        }

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
            httpc_test::new_client("http://127.0.0.1:8081").expect("Failed to create test client"),
        )
    }

    async fn get_test_client() -> Arc<Client> {
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

        response.print().await.ok();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.status(), StatusCode::OK);
    }
}
