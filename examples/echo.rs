use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    // Ok(0) が返ってきたらリモート側が閉じられたことを意味する
                    Ok(0) => {
                        println!("{} disconnected", socket.local_addr().unwrap());
                        return;
                    }
                    Ok(n) => {
                        // データをソケットへコピーする
                        println!("{} bytes read from client", n);
                        if socket.write_all(&buf[..n]).await.is_err() {
                            // ソケットへの書き込みが失敗したら接続を閉じる
                            println!("Failed to write to socket. Connection reset.");
                            return;
                        }
                    }
                    Err(_) => {
                        // ソケットからの読み込みが失敗したら接続を閉じる
                        println!("Failed to read from socket. Connection reset.");
                        return;
                    }
                }
            }
        });
    }
}
