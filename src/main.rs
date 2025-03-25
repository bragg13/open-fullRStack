use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use blogs_api::{create_blog, delete_blog, get_blog, get_blogs, update_blog};
use config::{get_db_url, AppStateInner};
use models::{Blog, BlogUpdatePayload};
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
        test_helper::{create_blogs_table, empty_blogs_table},
    };
    use axum::http::StatusCode;
    use httpc_test::{self, Client};
    use rstest::*;
    use serde_json::{json, Value};
    use std::{sync::Arc, time::Duration};
    use tokio::{net::TcpListener, sync::OnceCell, time::sleep};
    static TEST_CLIENT: OnceCell<Arc<Client>> = OnceCell::const_new();
    static TEST_SERVER: OnceCell<Arc<AppStateInner>> = OnceCell::const_new();

    async fn setup_client() -> Arc<Client> {
        Arc::new(
            httpc_test::new_client("http://127.0.0.1:8080").expect("Failed to create test client"),
        )
    }
    async fn setup_server() -> Arc<AppStateInner> {
        // Bind the server to a test port.
        let listener = TcpListener::bind("127.0.0.1:8080")
            .await
            .expect("Failed to bind to test port");

        // Create the state and app.
        let db_url = get_db_url(true);
        let state = Arc::new(AppStateInner::new(db_url).await);
        match create_blogs_table(&state.pool).await {
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

        // returns another state
        let db_url = get_db_url(true);
        Arc::new(AppStateInner::new(db_url).await)
    }

    #[fixture]
    async fn test_server() -> Arc<AppStateInner> {
        TEST_SERVER.get_or_init(setup_server).await.clone()
    }
    #[fixture]
    async fn test_client() -> Arc<Client> {
        TEST_CLIENT.get_or_init(setup_client).await.clone()
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

    // return value is an empty array if the database is empty
    // or returns 404 if single blog is requested
    #[rstest]
    #[case::get_blogs_1("/blogs/1", json!({"message": "Failed to retrieve blog"}), StatusCode::INTERNAL_SERVER_ERROR)]
    #[case::get_blogs("/blogs", json!([]), StatusCode::OK)]
    #[tokio::test]
    async fn get_blogs_empty_db(
        #[case] endpoint: &str,
        #[case] expected: Value,
        #[case] expected_status_code: StatusCode,
        #[future] test_server: Arc<AppStateInner>,
        #[future] test_client: Arc<Client>,
    ) {
        // let client = get_test_client().await;
        // let server = get_test_server().await;
        empty_blogs_table(&test_server.await.pool).await.ok();

        let response = test_client
            .await
            .do_get(endpoint)
            .await
            .expect("Expected to get a response from GET /blogs");

        let body = response.json_body().expect("Expected body to be defined");
        assert_eq!(expected, body);
        assert_eq!(expected_status_code, response.status());

        // necessary to wait for the server
        sleep(Duration::from_millis(200)).await;
    }

    // blogs are returned as json and are the correct amount
    // #[rstest]
    // #[case::single_blog("/blogs/1", json!([]))]
    // #[case::multiple_blogs("/blogs", json!([]))]
    // #[tokio::test]
    // async fn get_blogs_correct_json_fields(#[case] endpoint: &str, #[case] expected: Value) {
    //     let client = get_test_client().await;
    //     let server = get_test_server().await;

    //     empty_blogs_table(&server.pool).await.ok();
    //     let blogs = insert_test_values(&server.pool)
    //         .await
    //         .expect("Expected insert statement to work");
    //     println!("blogs: {:?}", blogs);

    //     let response = client
    //         .do_get(endpoint)
    //         .await
    //         .expect("Expected GET /blogs to work");

    //     let body = response.json_body().expect("Expected body to be defined");
    //     assert_eq!(expected, body)
    // }
    // TODO find a way to convert from &PgRow to Blog vector
    // let blogs: Vec<Blog> = blogs.iter().map(|el| Blog { id: todo!(), title: todo!(), author: todo!(), url: todo!(), likes: todo!() }}).collect();

    // let response = client
    //     .do_get("/blogs")
    //     .await
    //     .expect("Expected GET /blogs to return 6 blogs");

    // // get single blog given id - AND with wrong id AND missing ID
    // let response = client
    //     .do_get("/blogs")
    //     .await
    //     .expect("Expected GET /blogs to return []");

    // let body = response.json_body().expect("Expected body to be defined");
    // assert_eq!(json!([]), body);

    // let body = response.json_body().expect("Expected body to be defined");
    // assert_eq!(body.)

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
}
