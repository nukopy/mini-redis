use clap::Parser;
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,

    /// Port number to bind to
    #[arg(short, long, default_value = "6379")]
    port: u16,
}

#[tokio::main]
async fn main() {
    // Get args
    let args = Args::parse();

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
}

impl TcpListener {
    async fn new(addr: String) -> Self {
        // Initialize TCP listener to accept connections
        // Bind the listener to the address
        let listener = TokioTcpListener::bind(addr.clone()).await.unwrap();

        Self { addr, listener }
    }

    async fn run(&mut self) {
        loop {
            // println!("Waiting for a new connection...");
            // タプルの 2 つ目の要素は、新しいコネクションの IP とポートの情報を含んでいる
            let (socket, _socket_addr) = self.listener.accept().await.unwrap();

            // それぞれのインバウンドソケットに対して新しいタスクを spawn する。
            // ソケットは新しいタスクに move（所有権の移動）され、そこで処理される。
            tokio::spawn(async move {
                // println!("socket (TcpStream): {:?}", socket);
                // println!("Accepted connection from {:?}", socket_addr);
                TcpListener::process(socket).await;
            });
        }
    }

    async fn process(socket: TcpStream) {
        use mini_redis::Command::{self, Get, Set};
        use std::collections::HashMap;

        // データを蓄えるため、HashMap を使用する
        let mut db: HashMap<String, Vec<u8>> = HashMap::new();

        // `mini_redis` が提供する Connection によって、ソケットから来るフレームをパースする
        let mut connection = Connection::new(socket);

        while let Some(frame) = connection.read_frame().await.unwrap() {
            println!("GOT frame: {:?}", frame);

            // フレームをパースして、コマンドを取得する
            let response = match Command::from_frame(frame).unwrap() {
                Set(cmd) => {
                    db.insert(cmd.key().to_string(), cmd.value().to_vec());
                    println!("OK: Set (key, value) = ({}: {:?})", cmd.key(), cmd.value());
                    Frame::Simple("OK".to_string())
                }
                Get(cmd) => {
                    if let Some(value) = db.get(cmd.key()) {
                        // `Frame::Bulk` はデータが `Bytes` 型であることを期待する

                        Frame::Bulk(value.clone().into()) // into() を使用して、 `&Vec<u8>` から `Bytes` に変換する
                    } else {
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
/* mini_redis::Connection の実装
/// Send and receive `Frame` values from a remote peer.
///
/// When implementing networking protocols, a message on that protocol is
/// often composed of several smaller messages known as frames. The purpose of
/// `Connection` is to read and write frames on the underlying `TcpStream`.
///
/// To read frames, the `Connection` uses an internal buffer, which is filled
/// up until there are enough bytes to create a full frame. Once this happens,
/// the `Connection` creates the frame and returns it to the caller.
///
/// When sending frames, the frame is first encoded into the write buffer.
/// The contents of the write buffer are then written to the socket.
#[derive(Debug)]
pub struct Connection {
    // The `TcpStream`. It is decorated with a `BufWriter`, which provides write
    // level buffering. The `BufWriter` implementation provided by Tokio is
    // sufficient for our needs.
    stream: BufWriter<TcpStream>,

    // The buffer for reading frames.
    buffer: BytesMut,
}
 */
