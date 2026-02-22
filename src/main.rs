use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs, time::Duration};
use surge_ping;
use toml;

#[derive(Debug, Deserialize)]
struct Config {
    burst: Burst,
    targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
struct Burst {
    count: u8,
    timer: u8,
}

#[derive(Debug, Deserialize)]
struct Target {
    ip: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct Output {
    rtts: RTTs,
    target_name: String,
    target_ip: String,
    timestamp: DateTime<Local>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RTTs {
    min: f32,
    max: f32,
    avg: f32,
    loss: f32,
}

fn create_rtt(burst_res: &Vec<Duration>, expected: usize) -> RTTs {
    let received = burst_res.len();
    let loss = ((expected - received) as f32 / expected as f32) * 100.0;

    let min = burst_res.iter().min().unwrap().as_secs_f32() * 1000.0;
    let max = burst_res.iter().max().unwrap().as_secs_f32() * 1000.0;
    let avg = burst_res.iter().map(|d| d.as_secs_f32()).sum::<f32>() / received as f32 * 1000.0;

    RTTs {
        min,
        max,
        avg,
        loss,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "rtts.json";
    let config_str = fs::read_to_string("config.toml").expect("Failed to read file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse toml");
    for target in config.targets.iter() {
        let payload = [0; 8];
        let mut burst_res = Vec::new();
        for _n in 0..config.burst.count {
            let (_packet, duration) = surge_ping::ping(target.ip.parse()?, &payload).await?;
            burst_res.push(duration);
        }
        let rtt = create_rtt(&burst_res, config.burst.count as usize);
        let output = Output {
            rtts: rtt,
            target_name: target.name.clone(),
            target_ip: target.ip.clone(),
            timestamp: Local::now(),
        };
        println!(
            "[{}] - {}:({}) => RTT min:{:.3}ms, max:{:.3}ms, avg:{:.3}ms, loss:{:.3}ms",
            output.timestamp,
            output.target_name,
            output.target_ip,
            output.rtts.min,
            output.rtts.max,
            output.rtts.avg,
            output.rtts.loss
        );
        let mut file = fs::File::options()
            .append(true)
            .create(true)
            .open(filename)?;

        let _writeout = serde_json::to_writer_pretty(&mut file, &output);
    }

    println!("");
    Ok(())
}
