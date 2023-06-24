use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ArgsParser {
    /// IP address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    pub ip: String,

    /// Port number to bind to
    #[arg(short, long, default_value = "6379")]
    pub port: u16,
}
