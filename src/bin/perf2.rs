use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;
use env_logger::Builder;
use log::LevelFilter;
use solana_rpc_perf::utils::yellowstone::slot_subscribe_to_channel;
use tokio::sync::{Mutex, mpsc::{self, Receiver}};
use yellowstone_grpc_proto::geyser::CommitmentLevel;
use std::io::Write;
use chrono::Local;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    pub name_0: String,
    #[clap(long)]
    pub grpc_url_0: String,
    #[clap(long)]
    pub x_token_0: Option<String>,
    #[clap(short, long)]
    pub name_1: String,
    #[clap(short, long)]
    pub grpc_url_1: String,
    #[clap(short, long)]
    pub x_token_1: Option<String>,
}

// Shared state to track slot arrival times
struct SlotTracker {
    slot_times: Mutex<HashMap<u64, (String, Instant)>>
}

impl SlotTracker {
    fn new() -> Self {
        Self {
            slot_times: Mutex::new(HashMap::new())
        }
    }

    async fn track_slot(&self, slot: u64, endpoint_name: &str) {
        let mut slot_times = self.slot_times.lock().await;
        let current_time = Instant::now();

        if let Some((first_endpoint, first_time)) = slot_times.get(&slot) {
            let time_diff = current_time.duration_since(*first_time);
            log::info!("slot {} winner {}, {} ms faster", slot, first_endpoint, time_diff.as_millis());
            // log::info!(
            //     "Slot {} arrived at {} after {} ms (first arrival was by {})", 
            //     slot, 
            //     endpoint_name, 
            //     time_diff.as_millis(), 
            //     first_endpoint
            // );
        } else {
            slot_times.insert(slot, (endpoint_name.to_string(), current_time));
            // log::info!("First arrival of slot {} by {}", slot, endpoint_name);
        }
    }
}

async fn slot_monitor(
    endpoint_name: String, 
    mut receiver: Receiver<u64>, 
    slot_tracker: Arc<SlotTracker>
) {
    while let Some(slot) = receiver.recv().await {
        slot_tracker.track_slot(slot, &endpoint_name).await;
    }
}

fn subscribe(
    endpoint_name: String,
    grpc_url: String,
    x_token: Option<String>,
    slot_tracker: Arc<SlotTracker>
) -> (tokio::task::JoinHandle<()>, impl std::future::Future<Output = anyhow::Result<()>>) {
    let (tx, rx) = mpsc::channel(1000);
    let monitor = tokio::spawn(slot_monitor(
        endpoint_name.clone(), 
        rx, 
        slot_tracker.clone()
    ));

    let sub = slot_subscribe_to_channel(
        grpc_url,
        x_token,
        Arc::new(tx),
        CommitmentLevel::Processed,
    );

    (monitor, sub)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .init();

    // Create a shared slot tracker
    let slot_tracker = Arc::new(SlotTracker::new());

    let (monitor0, sub0) = subscribe(
        args.name_0.clone(), 
        args.grpc_url_0.clone(), 
        args.x_token_0.clone(),
        slot_tracker.clone()
    );

    let (monitor1, sub1) = subscribe(
        args.name_1.clone(), 
        args.grpc_url_1.clone(), 
        args.x_token_1.clone(),
        slot_tracker.clone()
    );

    // Run for desired duration
    tokio::select! {
        _ = sub0 => {},
        _ = sub1 => {},
        _ = tokio::signal::ctrl_c() => {
            log::info!("Received Ctrl+C, shutting down...");
        },
    }

    // Cleanup
    drop(monitor0);
    drop(monitor1);

    Ok(())
}