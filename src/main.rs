use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mini_redis::{
    Command::{self, Get, Set},
    Connection, Frame,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // Open a connection to the mini-redis address.
    let addr = String::from("127.0.0.1:6379");
    let tcp_server = TcpServer::new(addr);
    println!("Listening on {}", tcp_server.addr);

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
            println!("Accepted connection from {}", socket_addr);

            // それぞれのインバウンドソケットに対して、新しいタスクを生成 spawn する
            // ソケットは新しいタスクに move され、そこで処理がされる
            let db = self.db.clone();
            tokio::spawn(async move {
                TcpServer::process(db, socket).await; // variable `socket` moved here!
            });
        }
    }

    async fn process(db: Db, socket: TcpStream) {
        // `Connection` 型を使うことで、バイト列ではなく、Redis の「フレーム」を読み書きできるようになる。この `Connection` 型は mini-redis で定義されている。
        let mut connection = Connection::new(socket); // ソケットから来るフレームをパースする

        match connection.read_frame().await {
            Ok(Some(frame)) => {
                println!("GOT frame: {:?}", frame);

                // フレームをパースしてコマンドを実行する
                let frame = TcpServer::handle_frame(frame, db);
                let _ = connection.write_frame(&frame).await;
            }
            Ok(None) => {
                println!("Buffer is empty. Connection closed.");
            }
            Err(e) => {
                println!("Connection error: {:?}", e);
            }
        }
    }

    fn handle_frame(frame: Frame, db: Db) -> Frame {
        // フレームをパースして、コマンドを取得する
        match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                println!("SET {:?}", cmd);

                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                println!("GET {:?}", cmd);

                // DB から値を取り出す
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` はデータが `Bytes` 型であることを期待する
                    // .into() メソッドを使って、&Vec<u8> を `Bytes` に変換する
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
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
