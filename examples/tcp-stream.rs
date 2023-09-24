use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    // サーバに接続します（例: "127.0.0.1:12345" というアドレスのサーバに接続する場合）
    let mut socket = TcpStream::connect("127.0.0.1:8080").await?;
    let key = "hello ";

    // write to server
    println!("Writing to socket: {}", key);
    socket.write_all(key.as_bytes()).await?;

    // read from server
    let mut buffer = Vec::new();
    let n = socket.read_to_end(&mut buffer).await?;
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
