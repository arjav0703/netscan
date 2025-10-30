mod cli;
use anyhow::Result;
use futures::future::join_all;
use ipnetwork::IpNetwork;
use std::{net::IpAddr, sync::Arc};
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let config = cli::load_config();
    let network: IpNetwork = config.network.parse()?;

    println!("Scanning network: {network}");

    let ips = network.ips_in_network();
    println!("Possible IPs in the network: {}", ips.len());

    let up_hosts: Arc<tokio::sync::Mutex<Vec<IpAddr>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let tasks = ips.into_iter().map(|ip| {
        let up_hosts = up_hosts.clone();
        tokio::spawn(async move {
            if host_is_up(ip).await {
                // println!("{} responded to ping", ip);
                up_hosts.lock().await.push(ip);
            }
        })
    });

    let _ = join_all(tasks).await;

    let up_hosts = up_hosts.lock().await;
    println!("Hosts that are up:");
    for ip in up_hosts.iter() {
        println!("{}", ip);
    }

    Ok(())
}

trait IpNetworkExt {
    fn ips_in_network(&self) -> Vec<IpAddr>;
}

impl IpNetworkExt for IpNetwork {
    fn ips_in_network(&self) -> Vec<IpAddr> {
        match self {
            IpNetwork::V4(net) => net.iter().map(IpAddr::V4).collect(),
            IpNetwork::V6(net) => net.iter().map(IpAddr::V6).collect(),
        }
    }
}

async fn host_is_up(ip: IpAddr) -> bool {
    let config = Config::default();
    let client = match Client::new(&config) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut pinger = client.pinger(ip, PingIdentifier(0)).await;
    pinger.timeout(Duration::from_millis(800));

    pinger.ping(PingSequence(0), &[]).await.is_ok()
}
