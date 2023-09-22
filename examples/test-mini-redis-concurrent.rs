use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

use my_mini_redis::command::Command;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // サーバとのコネクションを確立する
    let addr = "127.0.0.1:6379";

    // constants
    let key = "hello";
    let value = "world";

    // 2 並行で Redis コマンドを実行する
    // チャネルに対してコマンドを送信するタスクを生成する
    let (tx, mut rx) = mpsc::channel(32); // 32 はチャネルのバッファサイズ
    let tx2 = tx.clone();

    // Get
    let task1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Get {
            key: key.to_string(),
            resp: resp_tx,
        };

        // manager タスクへコマンドを送信する
        tracing::info!("[task1 - Get] send command Get");

        tx2.send(cmd).await.unwrap();

        // manager タスクからのレスポンスを受信する
        let res = resp_rx.await;
        tracing::info!("[task1 - Get] Response = {:?}", res);
    });

    // Set
    let task2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: key.to_string(),
            value: value.into(),
            resp: resp_tx,
        };

        // manager タスクへコマンドを送信する
        tracing::info!("[task2 - Set] send command Set");
        tx.send(cmd).await.unwrap();

        // manager タスクからのレスポンスを受信する
        let res = resp_rx.await;
        tracing::info!("[task2 - Set] Response = {:?}", res);
    });

    // チャネルからのメッセージを処理するタスクを生成する
    let manager = tokio::spawn(async move {
        let mut client = client::connect(addr).await.unwrap(); // client を使いまわせるようになる
        tracing::info!("Connected to {}", addr);

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    tracing::info!("[manager] Get command received & send to server");
                    let res = client.get(&key).await;
                    let _ = resp.send(res); // エラーは無視する
                }
                Command::Set { key, value, resp } => {
                    tracing::info!("[manager] Set command received & send to server");
                    let res = client.set(&key, value).await;
                    let _ = resp.send(res); // エラーは無視する
                }
            }
        }
    });

    task1.await.unwrap();
    task2.await.unwrap();
    manager.await.unwrap();
}
