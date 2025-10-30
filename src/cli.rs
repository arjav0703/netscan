use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "netscan")]
#[command(about = "Discover hosts on a local network")]
pub struct Cli {
    #[arg(short, long)]
    pub network: String,
}

pub fn load_config() -> Cli {
    Cli::parse()
}
