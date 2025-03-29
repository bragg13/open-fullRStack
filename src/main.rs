use part4_bloglist::run;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    run().await?;
    Ok(())
}
