#[cfg(test)]
mod main_test {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use rstest::*;
    use serde_json::{json, Value};
    use sqlx::{migrate, postgres::PgPoolOptions};
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;
    use tower::ServiceExt;

    // async fn setup_client() -> Arc<Client> {
    //     Arc::new(
    //         httpc_test::new_client("http://127.0.0.1:8080").expect("Failed to create test client"),
    //     )
    // }

    // async fn setup_server() -> Arc<AppStateInner> {
    //     // Bind the server to a test port.
    //     let listener = TcpListener::bind("127.0.0.1:8080")
    //         .await
    //         .expect("Failed to bind to test port");

    //     // Create the state and app.
    //     let db_url = get_db_url(true);
    //     let state = Arc::new(AppStateInner::new(db_url).await);
    //     match create_blogs_table(&state.pool).await {
    //         Ok(_) => println!("DB initialised."),
    //         Err(e) => println!("Error: {e}"),
    //     }

    //     let app = app().await.with_state(state.clone());

    //     // Spawn the server in the background.
    //     tokio::spawn(async move {
    //         axum::serve(listener, app)
    //             .await
    //             .expect("Server failed during test");
    //     });

    //     // Give the server a moment to start up.
    //     sleep(Duration::from_millis(100)).await;

    //     state
    // }
    // async fn test_client() -> Arc<Client> {
    //     TEST_CLIENT.get_or_init(setup_client).await.clone()
    // }
    // async fn test_server() -> Arc<AppStateInner> {
    //     TEST_SERVER.get_or_init(setup_server).await.clone()
    // }

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
    #[tokio::test(flavor = "multi_thread")]
    async fn get_blogs_empty_db(
        #[case] endpoint: &str,
        #[case] expected: Value,
        #[case] expected_status_code: StatusCode,
    ) {
        // let db_url = format!(
        //     "postgres://postgres:potatocouch@127.0.0.1:{}/blogs-test",
        //     node.get_host_port_ipv4(5432).await.unwrap()
        // );

        let container = Postgres::default()
            .with_password("couchpotato")
            .with_user("postgres")
            .with_db_name("blogs-test")
            .start()
            .await
            .unwrap();

        let db_url = format!(
            "postgresql://postgres:couchpotato@localhost:{}/blogs-test",
            container.get_host_port_ipv4(5432).await.unwrap()
        );
        let db = PgPoolOptions::new()
            .connect(&format!(
                "postgresql://postgres:couchpotato@localhost:{}/blogs-test",
                container.get_host_port_ipv4(5432).await.unwrap()
            ))
            .await
            .unwrap();

        migrate!("./migrations").run(&db).await.unwrap();

        // assert_eq!(true, true);
        // let mut conn = postgres::Client::connect(connection_string, postgres::NoTls).unwrap();
        // let rows = conn.query("SELECT 1 + 1", &[]).unwrap();

        // let db_url = get_db_url(true);
        let pool = get_postgres_pool(db_url).await;
        let app = app(pool.clone()).await;

        // setup db
        // create_blogs_table(&pool)
        //     .await
        //     .expect("expected database to be initialised correctly");

        empty_blogs_table(&pool).await.ok();

        let server = TestServer::new(app).unwrap();
        let response = server.get(endpoint).await;
        let body: Value = response.json();

        response.assert_status(expected_status_code);
        assert_eq!(expected, body);
    }

    // blogs are returned as json and are the correct amount
    #[rstest]
    #[case::get_single_blog("/blogs/1", json!({
        "id": 1,
        "title": "React patterns",
                "author": "Michael Chan",
                "url": "https://reactpatterns.com/",
                "likes": 7,
            }), StatusCode::OK)]
    #[case::get_multiple_blogs("/blogs", json!([
        {
            "id": 1,
        "title": "React patterns".to_string(),
            "author": "Michael Chan".to_string(),
            "url": "https://reactpatterns.com/".to_string(),
            "likes": 7
        },
        {
            "id": 2,
        "title": "Go To Statement Considered Harmful".to_string(),
            "author": "Edsger W. Dijkstra".to_string(),
            "url": "http://blog.cleancoder.com/uncle-bob/2017/05/05/TestDefinitions.html".to_string(),
            "likes": 5
        },
        {
            "id": 3,
            "title": "Canonical string reduction".to_string(),
            "author": "Edsger W. Dijkstra".to_string(),
            "url": "http://www.u.arizona.edu/~rubinson/copyright_violations/Go_To_Considered_Harmful.html".to_string(),
            "likes": 12,
        },
        {
            "id": 4,
            "title": "TDD harms architecture".to_string(),
            "author": "Robert C. Martin".to_string(),
            "url": "http://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD808.html".to_string(),
            "likes": 10,
        },
        {
            "id": 5,
            "title": "Type wars".to_string(),
            "author": "Robert C. Martin".to_string(),
            "url": "http://blog.cleancoder.com/uncle-bob/2017/03/03/TDD-Harms-Architecture.html".to_string(),
            "likes": 0
        },
        {
            "id": 6,
            "title": "First class tests".to_string(),
            "author": "Robert C. Martin".to_string(),
            "url": "http://blog.cleancoder.com/uncle-bob/2016/05/01/TypeWars.html".to_string(),
            "likes": 2
        },

    ]), StatusCode::OK)]
    #[tokio::test]
    async fn get_blogs_correct_json_fields(
        #[case] endpoint: &str,
        #[case] expected: Value,
        #[case] expected_status_code: StatusCode,
    ) {
        use crate::test_helper::insert_test_values;

        let container = Postgres::default()
            .with_password("couchpotato")
            .with_user("postgres")
            .with_db_name("blogs-test")
            .start()
            .await
            .unwrap();

        let db_url = format!(
            "postgresql://postgres:couchpotato@localhost:{}/blogs-test",
            container.get_host_port_ipv4(5432).await.unwrap()
        );
        let db = PgPoolOptions::new()
            .connect(&format!(
                "postgresql://postgres:couchpotato@localhost:{}/blogs-test",
                container.get_host_port_ipv4(5432).await.unwrap()
            ))
            .await
            .unwrap();

        migrate!("./migrations").run(&db).await.unwrap();

        let pool = get_postgres_pool(db_url).await;
        let app = app(pool.clone()).await;

        // prepare database
        empty_blogs_table(&pool).await.ok();
        let blogs = insert_test_values(&pool)
            .await
            .expect("Expected insert statement to work");

        // perform request
        let server = TestServer::new(app).unwrap();
        let response = server.get(endpoint).await;
        let body: Value = response.json();

        response.assert_status(expected_status_code);
        assert_eq!(expected, body);
    }

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
