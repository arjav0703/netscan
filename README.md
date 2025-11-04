# NetScan
A simple network scanning tool to discover active devices on your local network.

## Installation
1. Clone the repository:
   ```
    git clone https://github.com/arjav0704/netscan.git
    cd netscan
    ```

2. Install Rust if you haven't already. You can download it from [here](https://rustup.rs).
3. Run the program using Cargo:
   ```
   cargo run -- --network <network_address>/<subnet_mask> --timeout <timeout_in_ms> --ports <comma_separated_ports>
   ```

## Supported CLI Arguments
- `--network <network_address>/<subnet_mask>`: Specify the network address and subnet mask to scan (e.g., `192.168.1.1/24`).
- `--timeout <timeout_in_ms>`: Set the timeout for each ping request in milliseconds (default is 1000 ms).
- `--ports <comma_separated_ports>`: Specify a list of ports to scan on each active device (e.g., `22,80,443`).
