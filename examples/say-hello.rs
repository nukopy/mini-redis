async fn say_world() {
    println!("world");
}

async fn say_hello_world() {
    let output = say_world();

    println!("hello");
    output.await;
}

async fn say_world_hello() {
    let output = say_world();

    output.await;
    println!("hello");
}

#[tokio::main]
async fn main() {
    say_hello_world().await;
    say_world_hello().await;
}
