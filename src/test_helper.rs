use axum::routing::get;
use sqlx::{database, postgres::PgRow, PgPool};

use crate::models::{Blog, BlogPostPayload};

/// Inserts 6 blogs in the database
///
/// # Returns
///
/// Returns a list of PgRows containing Blogs
pub async fn insert_test_values(pool: &PgPool) -> Result<Vec<Blog>, sqlx::Error> {
    // insert test entries
    let blogs = vec![
        BlogPostPayload {
            title: "React patterns".to_string(),
            author: "Michael Chan".to_string(),
            url: "https://reactpatterns.com/".to_string(),
            likes: Some(7),
        },
        BlogPostPayload {
            title: "Go To Statement Considered Harmful".to_string(),
            author: "Edsger W. Dijkstra".to_string(),
            url: "http://blog.cleancoder.com/uncle-bob/2017/05/05/TestDefinitions.html".to_string(),
            likes: Some(5),
        },
        BlogPostPayload {
            title: "Canonical string reduction".to_string(),
            author: "Edsger W. Dijkstra".to_string(),
            url: "http://www.u.arizona.edu/~rubinson/copyright_violations/Go_To_Considered_Harmful.html".to_string(),
            likes: Some(12),
        },
        BlogPostPayload {
            title: "TDD harms architecture".to_string(),
            author: "Robert C. Martin".to_string(),
            url: "http://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD808.html".to_string(),
            likes: Some(10),
        },
        BlogPostPayload {
            title: "Type wars".to_string(),
            author: "Robert C. Martin".to_string(),
            url: "http://blog.cleancoder.com/uncle-bob/2017/03/03/TDD-Harms-Architecture.html".to_string(),
            likes: Some(0),
        },
        BlogPostPayload {
            title: "First class tests".to_string(),
            author: "Robert C. Martin".to_string(),
            url: "http://blog.cleancoder.com/uncle-bob/2016/05/01/TypeWars.html".to_string(),
            likes: Some(2),
        },
    ];

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
        v_like.push(b.likes.unwrap_or(0));
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

pub async fn empty_blogs_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM TABLE if exists blogs")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_blogs_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    // drop the test table
    sqlx::query("DROP TABLE IF EXISTS blogs")
        .execute(pool)
        .await?;

    // create a new one
    sqlx::query(
        r#"
        CREATE TABLE blogs (
        id SERIAL PRIMARY KEY,
        title TEXT NOT NULL,
        author TEXT NOT NULL,
        url TEXT NOT NULL,
        likes INT NOT NULL DEFAULT 0,
        created_at TIMESTAMP DEFAULT NOW ()
    );
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
