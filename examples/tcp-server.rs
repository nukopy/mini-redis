use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // クライアントからの接続を待つ
        let (socket, socket_addr) = listener.accept().await?;
        println!("Accepted connection from: {}", socket_addr);

        // 各接続を非同期に処理
        tokio::spawn(async move {
            let _ = handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];

    // クライアントからのデータを読む
    println!("Reading from socket...");
    let n = socket.read(&mut buffer).await?;
    println!("{} bytes read from client", n);

    // response
    let response = format!("{}{}", String::from_utf8_lossy(&buffer), "world");
    println!("Writing to socket...");
    socket.write_all(response.as_bytes()).await?;

    Ok(())
}
