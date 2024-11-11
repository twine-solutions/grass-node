mod grass;

use clap::Parser;
use anyhow::{Context, Result};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::{Builder, Runtime};
use tokio::time::sleep;

#[derive(Parser)]
#[clap(author = "dropout", version = "1.0.0", about = "Grass Node via Proxies", long_about = None)]
struct Args {
    /// User ID
    #[clap(short, long, required = true)]
    user_id: String,

    /// Proxies file location
    #[clap(short, long, required = true)]
    proxies: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let args = Args::parse();

    log::info!("Let the chronicles begin... (user_id: {})\n", args.user_id);
    sleep(std::time::Duration::from_secs(2)).await;

    let runtime: Runtime = Builder::new_multi_thread()
        .max_blocking_threads(264)
        .worker_threads(264)
        .enable_all()
        .build()
        .unwrap();

    let file = File::open(args.proxies).await.context("Failed to open proxies file.")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut worker_num: i32 = 0;

    while let Some(proxy) = lines.next_line().await? {
        if proxy.trim().is_empty() || proxy.starts_with('#') {
            continue;
        }

        worker_num += 1;
        runtime.spawn(worker(worker_num, args.user_id.clone(), proxy.clone()));
    }

    loop {}
}

async fn worker(num: i32, user_id: String, proxy: String) {
    let log_target = format!("worker-{}", num);

    loop {
        let mut grass = match grass::Grass::new(log_target.clone(), user_id.clone(), Some(proxy.as_str())) {
            Ok(g) => g,
            Err(e) => {
                log::error!(target: &log_target, "{}", e);
                continue;
            }
        };
        log::info!(target: &log_target, "Connecting to Grass (device_id: {})", grass.device_id);

        match grass.connect().await {
            Ok(_) => {},
            Err(e) => log::error!(target: &log_target, "Connection error: {}", e)
        }
    }
}