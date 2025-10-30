mod cli;
use std::net::IpAddr;

use anyhow::Result;
use ipnetwork::IpNetwork;

#[tokio::main]
async fn main() -> Result<()> {
    let config = cli::load_config();
    let network: IpNetwork = config.network.parse()?;

    println!("Scanning network: {network}");

    println!(
        "Possible IPs in the network: {}",
        network.ips_in_network().len()
    );

    Ok(())
}

trait IpNetworkExt {
    fn ips_in_network(&self) -> Vec<IpAddr>;
}

impl IpNetworkExt for IpNetwork {
    fn ips_in_network(&self) -> Vec<IpAddr> {
        self.iter().collect()
    }
}
