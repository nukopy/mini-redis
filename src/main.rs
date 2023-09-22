use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mini_redis::{
    Command::{self, Get, Set},
    Connection, Frame,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Open a connection to the mini-redis address.
    let addr = String::from("127.0.0.1:6379");
    let tcp_server = TcpServer::new(addr);
    tracing::info!("Listening on {}", tcp_server.addr);

    // Run the TCP server
    let output = tcp_server.run();
    output.await;
}

type DbInternal = HashMap<String, Vec<u8>>;
type Db = Arc<Mutex<DbInternal>>;

struct TcpServer {
    addr: String,
    db: Db,
}

impl TcpServer {
    fn new(addr: String) -> Self {
        let db = Arc::new(Mutex::new(HashMap::new()));
        Self { addr, db }
    }

    async fn run(&self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();

        loop {
            // タプルの 2 つ目の要素は、新しいコネクションの IP とポートの情報を含んでいる
            let (socket, socket_addr) = listener.accept().await.unwrap();
            tracing::info!("Accepted connection from {}", socket_addr);

            // それぞれのインバウンドソケットに対して、新しいタスクを生成 spawn する
            // ソケットは新しいタスクに move され、そこで処理がされる
            let db = self.db.clone();
            tokio::spawn(async move {
                TcpServer::process(socket, db).await; // variable `socket` moved here!
            });
        }
    }

    async fn process(socket: TcpStream, db: Db) {
        // `Connection` 型を使うことで、バイト列ではなく、Redis の「フレーム」を読み書きできるようになる。この `Connection` 型は mini-redis で定義されている。
        let mut connection = Connection::new(socket); // ソケットから来るフレームをパースする

        while let Ok(Some(frame)) = connection.read_frame().await {
            tracing::info!("GOT frame: {:?}", frame);

            // フレームをパースしてコマンドを実行する
            let response = TcpServer::handle_frame(frame, db.clone());
            if let Err(e) = connection.write_frame(&response).await {
                tracing::error!("Failed to write frame: {:?}", e);
                return;
            }
        }
    }

    fn handle_frame(frame: Frame, db: Db) -> Frame {
        // フレームをパースして、コマンドを取得する
        match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                tracing::info!("SET {:?}", cmd);

                match db.lock() {
                    Ok(mut db) => {
                        // DB に値をセットする
                        tracing::info!("SET: key={:?}, value={:?}", cmd.key(), cmd.value());
                        db.insert(cmd.key().to_string(), cmd.value().to_vec());
                        Frame::Simple("OK".to_string())
                    }
                    Err(err) => {
                        tracing::error!("lock error: {:?}", err);
                        Frame::Error("lock error".to_string())
                    }
                }
            }
            Get(cmd) => {
                tracing::info!("GET {:?}", cmd);

                // DB から値を取り出す
                match db.lock() {
                    Ok(db) => match db.get(cmd.key()) {
                        Some(value) => {
                            tracing::info!(
                                "GET: key={:?}, value={:?}",
                                cmd.key(),
                                db.get(cmd.key())
                            );
                            Frame::Bulk(value.clone().into())
                        }
                        None => {
                            tracing::info!("GET: No value found for key {:?}", cmd.key());
                            Frame::Null
                        }
                    },
                    Err(err) => {
                        tracing::error!("lock error: {:?}", err);
                        Frame::Error("lock error".to_string())
                    }
                }
            }
            _ => Frame::Error("unimplemented".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[should_panic]
    fn it_works2() {
        panic!("Make this test fail");
    }
}
