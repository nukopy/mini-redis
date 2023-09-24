use clap::Parser;

use my_mini_redis::{args_parser::ArgsParser, server::MiniRedisServer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Define server
    let args = ArgsParser::parse();
    let addr = format!("{}:{}", args.ip, args.port);
    let server = MiniRedisServer::new(addr);

    // Run server
    let output = server.run();
    output.await;
}
