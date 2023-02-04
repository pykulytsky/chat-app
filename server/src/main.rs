use server::Server;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = Server::bind("127.0.0.1:9999").await.unwrap();
    server.run().await?;
    Ok(())
}
