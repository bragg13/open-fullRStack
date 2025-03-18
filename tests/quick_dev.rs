use anyhow::Result;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;
    hc.do_get("/blogs").await?.print().await?;
    hc.do_get("/blogs/1").await?.print().await?;
    Ok(())
}
