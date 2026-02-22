use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};
use surge_ping::{Client, Config, PingIdentifier, PingSequence};

#[derive(Debug, Deserialize)]
struct Conf {
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
    mid: f32,
    loss: f32,
}

fn median(burst_res: &mut [Duration]) -> f32 {
    burst_res.sort();
    let mid = burst_res.len() / 2;
    if burst_res.len().is_multiple_of(2) {
        (burst_res[mid - 1].as_secs_f32() + burst_res[mid].as_secs_f32()) / 2.0 * 1000.0
    } else {
        burst_res[mid].as_secs_f32() * 1000.0
    }
}

fn create_rtt(burst_res: &mut [Duration], expected: usize) -> RTTs {
    if burst_res.is_empty() {
        return RTTs {
            min: -1.0,
            max: -1.0,
            avg: -1.0,
            mid: -1.0,
            loss: 100.0,
        };
    }
    let received = burst_res.len();
    let loss = ((expected - received) as f32 / expected as f32) * 100.0;

    let min = burst_res.iter().min().unwrap().as_secs_f32() * 1000.0;
    let max = burst_res.iter().max().unwrap().as_secs_f32() * 1000.0;
    let avg = burst_res.iter().map(|d| d.as_secs_f32()).sum::<f32>() / received as f32 * 1000.0;

    let mid = median(burst_res);

    RTTs {
        min,
        max,
        avg,
        mid,
        loss,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "rtts.json";
    let config_str = fs::read_to_string("config.toml").expect("Failed to read file");
    let config: Conf = toml::from_str(&config_str).expect("Failed to parse toml");
    loop {
        for target in config.targets.iter() {
            let payload = [0; 8];
            let mut burst_res = Vec::new();
            for n in 0..config.burst.count {
                let client = Client::new(&Config::default())?;
                let mut pinger = client.pinger(target.ip.parse()?, PingIdentifier(0)).await;
                pinger.timeout(Duration::from_secs(1));
                if let Ok((_packet, duration)) = pinger.ping(PingSequence(n as u16), &payload).await
                {
                    burst_res.push(duration)
                }
            }
            let rtt = create_rtt(&mut burst_res, config.burst.count as usize);
            let output = Output {
                rtts: rtt,
                target_name: target.name.clone(),
                target_ip: target.ip.clone(),
                timestamp: Local::now(),
            };
            println!(
                "[{}] - {}:({}) => RTT min:{:.3}ms, max:{:.3}ms, avg:{:.3}ms, mid:{:.3}ms, loss:{:.0}%",
                output.timestamp,
                output.target_name,
                output.target_ip,
                output.rtts.min,
                output.rtts.max,
                output.rtts.avg,
                output.rtts.mid,
                output.rtts.loss
            );
            let mut file = fs::File::options()
                .append(true)
                .create(true)
                .open(filename)?;

            let _writeout = serde_json::to_writer_pretty(&mut file, &output);
        }

        println!();
        tokio::time::sleep(Duration::from_secs(config.burst.timer as u64)).await;
    }
    #[allow(unreachable_code)]
    Ok(())
}
