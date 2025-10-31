use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "netscan")]
#[command(about = "Discover hosts on a local network")]
pub struct Cli {
    #[arg(short, long)]
    pub network: String,

    #[arg(short, long, default_value_t = 1000)]
    pub timeout: u64,

    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Ports to scan (comma-separated). Default: 21,22,23,25,53,80,110,143,443,445,3306,3389,5432,5900,8080,8443"
    )]
    pub ports: Option<Vec<u16>>,
}

impl Cli {
    pub fn get_ports(&self) -> Vec<u16> {
        self.ports.clone().unwrap_or_else(|| {
            vec![
                21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 3306, 3389, 5432, 5900, 8080, 8443,
            ]
        })
    }
}

pub fn load_config() -> Cli {
    Cli::parse()
}
