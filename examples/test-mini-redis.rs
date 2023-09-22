use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let addr = "127.0.0.1:6379";

    // Set the key "hello" with value "world"
    let key = "hello";
    let value = "world";
    println!("Setting key \"{}\" to \"{}\".", key, value);
    let mut client = client::connect(addr).await?;
    client.set(key, value.into()).await?;

    // Get key "hello"
    let mut client = client::connect(addr).await?;
    let result = client.get(key).await?;

    println!("Got value from mini-redis server; result = {:?}", result);

    Ok(())
}
