use anyhow::Result;
use tokio;
use server::http_server;

#[tokio::main]
async fn main() -> Result<()> {
    let hostname = "localhost".to_string();
    let port = 8080;
    http_server::start_server(hostname, port).await?;
    Ok(())
}