/*
1. ある時間、待機する
2. 標準出力に特定のテキストを出力する
3. 文字列を返す
 */

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

// std::future::Future トレイトを実装する
// pub trait Future {
//     type Output;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
// }

struct Delay {
    when: Instant,
    poll_cnt: u32,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        // println!("polling...");
        // mutate poll_cnt
        let this = self.get_mut();
        this.poll_cnt += 1;
        if Instant::now() >= this.when {
            println!("poll_cnt: {}", this.poll_cnt);
            Poll::Ready("done")
        } else {
            // 今はこの行は無視してください
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Start simple future");
    let when = Instant::now() + Duration::from_secs(3);
    let future = Delay { when, poll_cnt: 0 };

    let out = future.await;
    assert_eq!(out, "done");
}
