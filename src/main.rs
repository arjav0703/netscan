mod cli;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = cli::load_config();
    println!("Scanning network: {}", config.network);

    Ok(())
}
