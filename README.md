# spingc

A lightweight network diagnostic tool inspired by SmokePing. Sends ICMP burst pings to multiple targets, calculates RTT statistics, and logs structured NDJSON output for later analysis.

Built to diagnose intermittent ISP connectivity issues — specifically to identify whether packet loss and latency spikes occur at the local gateway, ISP edge, or further upstream.

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
timer = 60

[[targets]]
ip = "192.168.1.1"
name = "Local GW"

[[targets]]
ip = "168.192.1.1"
name = "ISP GW"

[[targets]]
ip = "172.217.19.238"
name = "google.com"
​```

| Field | Description |
|---|---|
| `burst.count` | Number of pings per burst |
| `burst.timer` | Interval in seconds between bursts |
| `targets.ip` | Target IP (FQDN not supported) |
| `targets.name` | Human-readable label |

## Usage
```bash
./spingc
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
