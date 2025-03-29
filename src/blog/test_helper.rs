#[cfg(test)]
pub mod test_helper {
    use sqlx::migrate;
    use sqlx::PgPool;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::postgres::Postgres;
    use tokio::net::TcpListener;

    use crate::app;
    use crate::models::Blog;

    /// TODO: change this to a migration??
    /// Inserts 6 blogs in the database
    ///
    /// # Returns
    ///
    /// Returns a list of PgRows containing Blogs
    pub async fn insert_test_values(pool: &PgPool) -> Result<Vec<Blog>, sqlx::Error> {
        // insert test entries
        let blogs = get_test_blogs();
        let (mut v_title, mut v_author, mut v_url, mut v_like): (
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<i32>,
        ) = (vec![], vec![], vec![], vec![]);

        blogs.into_iter().for_each(|b| {
            v_title.push(b.title);
            v_author.push(b.author);
            v_url.push(b.url);
            v_like.push(b.likes);
        });

        let res = sqlx::query_as!(
            Blog,
            "
        INSERT INTO blogs (title, author, url, likes)
            SELECT * FROM
            UNNEST($1::text[], $2::text[], $3::text[], $4::int[])
            returning id, title, author, url, likes",
            &v_title[..],
            &v_author[..],
            &v_url[..],
            &v_like[..]
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }

    pub fn get_test_blogs() -> Vec<Blog> {
        vec![
        Blog{
            id: 1,
            title: "React patterns".to_string(),
            author: "Michael Chan".to_string(),
            url: "https://reactpatterns.com/".to_string(),
            likes: 7,
        },
        Blog{
            id: 2,
            title: "Go To Statement Considered Harmful".to_string(),
            author: "Edsger W. Dijkstra".to_string(),
            url: "http://blog.cleancoder.com/uncle-bob/2017/05/05/TestDefinitions.html".to_string(),
            likes: 5,
        },
        Blog{
            id: 3,
            title: "Canonical string reduction".to_string(),
            author: "Edsger W. Dijkstra".to_string(),
            url: "http://www.u.arizona.edu/~rubinson/copyright_violations/Go_To_Considered_Harmful.html".to_string(),
            likes: 12,
        },
        Blog{
            id: 4,
            title: "TDD harms architecture".to_string(),
            author: "Robert C. Martin".to_string(),
            url: "http://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD808.html".to_string(),
            likes: 10,
        },
        Blog{
            id: 5,
            title: "Type wars".to_string(),
            author: "Robert C. Martin".to_string(),
            url: "http://blog.cleancoder.com/uncle-bob/2017/03/03/TDD-Harms-Architecture.html".to_string(),
            likes: 0,
        },
        Blog{
            id: 6,
            title: "First class tests".to_string(),
            author: "Robert C. Martin".to_string(),
            url: "http://blog.cleancoder.com/uncle-bob/2016/05/01/TypeWars.html".to_string(),
            likes: 2,
        },
    ]
    }
    pub struct TestServer {
        pub address: String,
        pub db_pool: PgPool,
        pub container: ContainerAsync<Postgres>,
    }

    pub async fn spawn_app() -> TestServer {
        // address to bind - with random port!
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        // create test container
        let container = Postgres::default().start().await.unwrap();
        let db_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            container.get_host_port_ipv4(5432).await.unwrap()
        );
        let db_pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to connect to database");

        // connect to db pool
        migrate!("./migrations").run(&db_pool).await.unwrap();

        // create server
        let app = app(db_pool.clone()).await;
        let _ = tokio::spawn(async { axum::serve(listener, app).await });
        // tokio::spawn(async move { axum::serve(listener, app.into_make_service()).await });
        println!("test server running at {:?}", address);
        TestServer {
            address,
            db_pool,
            container,
        }
    }
}
