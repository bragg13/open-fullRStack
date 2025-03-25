use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

// A state is a struct that implements Clone.
// It has to implement Clone because Axum clones it for every handler call.
pub struct AppStateInner {
    pub pool: PgPool,
}

impl AppStateInner {
    pub async fn new(db_url: String) -> Self {
        println!("{}", db_url);
        let pool = PgPoolOptions::new()
            .connect(&db_url)
            .await
            .expect("Error connecting to database");
        info!("Connected to Database.");
        Self { pool }
    }
}

pub fn get_db_url(is_test: bool) -> String {
    dotenv().ok();
    let mut db_name = std::env::var("DB_NAME").expect("Could not read env variable DB_NAME");
    if is_test {
        db_name = db_name + "-test";
    }
    let db_psw = std::env::var("DB_PASSWORD").expect("Could not read env variable DB_PASSWORD");
    let db_user = std::env::var("DB_USER").expect("Could not read env variable DB_USER");
    let port = if is_test { 5433 } else { 5432 };

    format!("postgres://{db_user}:{db_psw}@localhost:{port}/{db_name}")
}
