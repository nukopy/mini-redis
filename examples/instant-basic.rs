use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    // 現在の時刻を取得
    let now = Instant::now();
    let earlier = now - Duration::from_secs(10); // Instant オブジェクト同士の引き算は、経過時間を表す Duration オブジェクトを返す
    if earlier < now {
        println!("Earlier time is indeed earlier than now!");
    }

    // 時間のかかる処理
    time_consuming_task();

    // 経過時間を表示
    let duration = now.elapsed();
    println!("Elapsed time: {:?}", duration);
    println!("Elapsed time: {:?}", duration.as_nanos());
    println!("Elapsed time: {:?}", duration.as_micros());
    println!("Elapsed time: {:?}", duration.as_millis());
    println!("Elapsed time: {:?}", duration.as_secs());

    let later = duration + Duration::from_secs(10);
    println!("Later time: {:?}", later);
    println!("Later time: {:?}", later.as_nanos());
    println!("Later time: {:?}", later.as_micros());
    println!("Later time: {:?}", later.as_millis());
    println!("Later time: {:?}", later.as_secs());
}

fn time_consuming_task() {
    println!("Start time consuming task");
    sleep(std::time::Duration::from_secs(1));
}
