use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let blog1 = json!({
        "author": "andrea",
        "title": "blog prova 1",
        "url": "www.bing.org",
        "likes": 17
    });
    let blog2 = json!({
        "author": "andrea",
        "title": "blog prova 2",
        "url": "www.google.it",
        "likes": 8
    });
    let hc = httpc_test::new_client("http://localhost:8080")?;

    // create two blogs
    hc.do_post("/blogs", blog1).await?.print().await?;
    hc.do_post("/blogs", blog2).await?.print().await?;

    // get all blogs
    hc.do_get("/blogs").await?.print().await?;

    // get a single blog
    hc.do_get("/blogs/1").await?.print().await?;

    // update a single blog
    // hc.do_put("/blogs/1", blog.).await?.print().await?;

    // delete  each blog
    hc.do_delete("/blogs/1").await?.print().await?;
    hc.do_delete("/blogs/2").await?.print().await?;
    Ok(())
}
