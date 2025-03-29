use configuration::get_configuration;
use part4_bloglist::run;
use sqlx::PgPool;
use tracing::{info, Level};
mod configuration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // connect to database
    let configuration = get_configuration().expect("Failed to read configuration file");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");

    // run application
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    info!("Application running at {}", &addr);

    // run(listener, db_pool).await?;
    tokio::spawn(run(listener, db_pool));
    Ok(())
}
