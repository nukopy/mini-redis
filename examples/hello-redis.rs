use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let addr = "127.0.0.1:6379";
    println!("Connecting to {}", addr);
    let mut client = client::connect(addr).await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    // let mut client = client::connect(addr).await?;
    let result = client.get("hello").await?;

    println!(
        "Got value from the mini-redis server; result = {:?}",
        result
    );

    Ok(())
}

/* cargo expand --example hello-tokio
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use mini_redis::{client, Result};
fn main() -> Result<()> {
    let body = async {
        let addr = "127.0.0.1:6379";
        let mut client = client::connect(addr).await?;
        client.set("hello", "world".into()).await?;
        let result = client.get("hello").await?;
        {
            ::std::io::_print(format_args!(
                "Got value from the mini-redis server; result = {0:?}\n",
                result
            ));
        };
        Ok(())
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
*/

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
