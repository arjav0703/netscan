mod cli;
use anyhow::Result;
use dns_lookup::lookup_addr;
use futures::future::join_all;
use ipnetwork::IpNetwork;
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Instant,
};
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

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
    println!("\n{} host(s) discovered:\n", up_hosts.len());
    println!(
        "{:<15} {:<30} {:<8} {}",
        "IP Address", "Hostname", "Latency", "Open Ports"
    );
    println!("{}", "-".repeat(80));

    let ports = config.get_ports();
    let timeout = Duration::from_millis(config.timeout);

    for ip in up_hosts.iter() {
        let info = get_host_info(*ip, ports.clone(), timeout).await;
        println!("{}", info);
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
    pinger.timeout(Duration::from_millis(1000));

    pinger.ping(PingSequence(0), &[]).await.is_ok()
}

#[derive(Debug)]
struct HostInfo {
    ip: IpAddr,
    hostname: Option<String>,
    latency_ms: f64,
    open_ports: Vec<u16>,
}

impl std::fmt::Display for HostInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hostname = self.hostname.as_deref().unwrap_or("Unknown");
        let ports = if self.open_ports.is_empty() {
            "None".to_string()
        } else {
            self.open_ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        write!(
            f,
            "{:<15} {:<30} {:<8.2} {}",
            self.ip, hostname, self.latency_ms, ports
        )
    }
}

async fn get_host_info(ip: IpAddr, ports: Vec<u16>, timeout: Duration) -> HostInfo {
    let hostname = lookup_hostname(ip, timeout).await;
    let latency_ms = measure_latency(ip, timeout).await;
    let open_ports = scan_common_ports(ip, ports, timeout).await;

    HostInfo {
        ip,
        hostname,
        latency_ms,
        open_ports,
    }
}

async fn lookup_hostname(ip: IpAddr, _timeout: Duration) -> Option<String> {
    tokio::task::spawn_blocking(move || lookup_addr(&ip).ok())
        .await
        .ok()
        .flatten()
}

async fn measure_latency(ip: IpAddr, timeout: Duration) -> f64 {
    let config = Config::default();
    let client = match Client::new(&config) {
        Ok(c) => c,
        Err(_) => return 0.0,
    };

    let mut pinger = client.pinger(ip, PingIdentifier(0)).await;
    pinger.timeout(timeout);

    let start = Instant::now();
    match pinger.ping(PingSequence(0), &[]).await {
        Ok(_) => start.elapsed().as_secs_f64() * 1000.0,
        Err(_) => 0.0,
    }
}

async fn scan_common_ports(ip: IpAddr, ports: Vec<u16>, timeout: Duration) -> Vec<u16> {
    let mut open_ports = Vec::new();
    let mut tasks = Vec::new();

    for port in ports {
        let task = tokio::spawn(async move {
            if is_port_open(ip, port, timeout).await {
                Some(port)
            } else {
                None
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        if let Ok(Some(port)) = task.await {
            open_ports.push(port);
        }
    }

    open_ports.sort();
    open_ports
}

async fn is_port_open(ip: IpAddr, port: u16, timeout_dur: Duration) -> bool {
    let addr = SocketAddr::new(ip, port);
    match timeout(timeout_dur, TcpStream::connect(addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}
