mod args_parser;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use clap::Parser;
use mini_redis::{
    Command::{self, Get, Set},
    Connection, Frame,
};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};

use args_parser::ArgsParser;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    // Get args
    let args = ArgsParser::parse();

    // Initialize TCP server
    let addr = format!("{}:{}", args.ip, args.port);
    let mut tcp_server = TcpListener::new(addr).await;
    println!("Listening on {}", tcp_server.addr);

    // Start listening for incoming connections
    // tcp_server.run();
    tcp_server.run().await;
    println!("End of main");
}

struct TcpListener {
    addr: String,
    listener: TokioTcpListener,
    db: Db,
}

impl TcpListener {
    async fn new(addr: String) -> Self {
        // Initialize TCP listener to accept connections
        // Bind the listener to the address
        let listener = TokioTcpListener::bind(addr.clone()).await.unwrap();
        let db: Db = Arc::new(Mutex::new(HashMap::new()));

        Self { addr, listener, db }
    }

    async fn run(&mut self) {
        loop {
            // println!("Waiting for a new connection...");
            // タプルの 2 つ目の要素は、新しいコネクションの IP とポートの情報を含んでいる
            let (socket, _socket_addr) = self.listener.accept().await.unwrap();

            // それぞれのインバウンドソケットに対して新しいタスクを spawn する。
            // ソケットは新しいタスクに move（所有権の移動）され、そこで処理される。
            let db = self.db.clone();
            tokio::spawn(async move {
                // println!("socket (TcpStream): {:?}", socket);
                // println!("Accepted connection from {:?}", socket_addr);
                TcpListener::process(socket, db).await;
            });
        }
    }

    async fn process(socket: TcpStream, db: Db) {
        // `mini_redis` が提供する Connection によって、ソケットから来るフレームをパースする
        let mut connection = Connection::new(socket);

        if let Some(frame) = connection.read_frame().await.unwrap() {
            println!("GOT frame: {:?}", frame);

            // フレームをパースして、コマンドを取得する
            let response = match Command::from_frame(frame).unwrap() {
                Set(cmd) => {
                    let mut db = db.lock().unwrap();
                    db.insert(cmd.key().to_string(), cmd.value().clone());
                    println!("OK: Set (key, value) = ({}: {:?})", cmd.key(), cmd.value());
                    Frame::Simple("OK".to_string())
                }
                Get(cmd) => {
                    let db = db.lock().unwrap();
                    if let Some(value) = db.get(cmd.key()) {
                        // `Frame::Bulk` はデータが `Bytes` 型であることを期待する
                        println!("OK: Get (key, value) = ({}: {:?})", cmd.key(), value);
                        Frame::Bulk(value.clone())
                    } else {
                        println!("OK: No value for key \"{}\"", cmd.key());
                        Frame::Null
                    }
                }
                cmd => panic!("unimplemented {:?}", cmd),
            };

            // クライアントへのレスポンスを書き込む
            connection.write_frame(&response).await.unwrap();
        }
    }
}
