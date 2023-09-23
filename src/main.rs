use my_mini_redis::server::MiniRedisServer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Define server
    let addr = String::from("127.0.0.1:6379");
    let server = MiniRedisServer::new(addr);

    // Run server
    let output = server.run();
    output.await;
}
