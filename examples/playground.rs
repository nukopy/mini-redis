use std::rc::Rc;
use tokio::task::yield_now;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        // { } で囲っていることにより、`rc` が `.await` の前に drop される
        {
            let rc = Rc::new("hello");
            println!("{}", rc);
        }

        // `rc` はもはや使用されない。
        // タスクがスケジューラに戻るときには破棄されている
        yield_now().await;
    });
}
