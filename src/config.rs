use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

// A state is a struct that implements Clone.
// It has to implement Clone because Axum clones it for every handler call.
pub struct AppStateInner {
    pub pool: PgPool,
}

impl AppStateInner {
    pub async fn new() -> Self {
        let db_url =
            std::env::var("DATABASE_URL").expect("Could not read env variable DATABASE_URL");
        let pool = PgPoolOptions::new()
            .connect(&db_url)
            .await
            .expect("Error connecting to database");
        info!("Connected to Database.");
        Self { pool }
    }
}
