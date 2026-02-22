# spingc

A lightweight and perhaps fugly network diagnostic tool inspired by SmokePing. 
Sends ICMP burst pings to multiple targets, calculates RTT statistics, and logs structured NDJSON output for later analysis.

Built to diagnose intermittent ISP connectivity issues — specifically to identify whether packet loss and latency spikes occur at the local gateway, ISP edge, or further upstream.

This is a tool that is a learning path to rust knowladge for me..

## Features

- Configurable burst pinging (N pings per target per interval)
- Per-target statistics: min, max, avg, median RTT and packet loss
- PTR (reverse DNS) lookup for each target
- NDJSON output for easy parsing and future visualization
- Runs continuously with configurable interval
- Single binary, no runtime dependencies
- Cross-compiles for Windows

## Configuration

Create a `config.toml` in the same directory as the binary:

```toml
[burst]
count = 10
timer = 20

[[targets]]
ip = "192.168.1.1"
name = "Local GW"

[[targets]]
ip = "168.192.1.1"
name = "ISP GW"

[[targets]]
ip = "172.217.19.238"
name = "google.com"
```

| Field | Description |
|---|---|
| `burst.count` | Number of pings per burst |
| `burst.timer` | Interval in seconds between bursts |
| `targets.ip` | Target IP (FQDN not supported) |
| `targets.name` | Human-readable label |

## Finding your network path

Before configuring targets, map your route to identify interesting hops to monitor.

### Windows
```
tracert 8.8.8.8
```

Example output:
```
Tracing route to dns.google [8.8.8.8] over a maximum of 30 hops:

  1    <1 ms    <1 ms    <1 ms  192.168.1.1
  2     *        *        *     Request timed out.
  3     1 ms     1 ms     1 ms  gw1-customer.isp-example.net [83.24.110.1]
  4     4 ms     5 ms     4 ms  core1-sto.isp-example.net [83.24.100.5]
  5     2 ms     2 ms     2 ms  peer1-sto.upstream-net.net [91.200.16.1]
  6     2 ms     2 ms     3 ms  dns.google [8.8.8.8]
```

### Linux/macOS
```bash
mtr --report 8.8.8.8
```

Pick the interesting hops as targets—typically your local gateway, your ISP edge router, and one or two upstream hops. Skip hops that show `* * *` (they block ICMP but still forward traffic).

### Example config.toml based on the route above
```toml
[burst]
count = 10        # Send a burst of 10 pings per interval
timer = 60        # Interval in seconds between bursts

[[targets]]
ip = "192.168.1.1"
name = "Local GW"

[[targets]]
ip = "83.24.110.1"
name = "gw1-customer.isp-example.net"

[[targets]]
ip = "83.24.100.5"
name = "core1-sto.isp-example.net"

[[targets]]
ip = "91.200.16.1"
name = "peer1-sto.upstream-net.net"

[[targets]]
ip = "8.8.8.8"
name = "Google DNS"
```

## Usage

Linux:

```bash
./spingc

```
Windows terminal:
```powershell
.\spingc.exe
```

Runs continuously. Results are appended to `results.ndjson` in the current directory.

## Output

Console:
```
[2026-02-20 14:34:23 +01:00] - Local GW:(192.168.1.1) => RTT min:0.087ms, max:0.203ms, avg:0.114ms, loss:0.000%
[2026-02-20 14:34:23 +01:00] - google.com:(172.217.19.238) => RTT min:2.460ms, max:2.524ms, avg:2.499ms, loss:0.000%
```

NDJSON (`results.ndjson`):
```json
{"timestamp":"2026-02-20T14:34:23.221+01:00","target_name":"Local GW","target_ip":"192.168.1.1","rtts":{"min":0.087,"max":0.203,"avg":0.114,"median":0.110,"loss":0.0}}
```

## Building

### Linux
```bash
cargo build --release
```

### Cross-compile for Windows
```bash
cargo install cross
cross build --target x86_64-pc-windows-gnu --release
```

Binary: `target/x86_64-pc-windows-gnu/release/spingc.exe`

## Linux: Unprivileged ICMP
```bash
sudo sysctl -w net.ipv4.ping_group_range="0 2147483647"
```

Or set capability on the binary:
```bash
sudo setcap cap_net_raw+ep ./spingc
```

## Dependencies

- [surge-ping](https://crates.io/crates/surge-ping) — async ICMP
- [tokio](https://crates.io/crates/tokio) — async runtime
- [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) — serialization
- [toml](https://crates.io/crates/toml) — config parsing
- [chrono](https://crates.io/crates/chrono) — timestamps
