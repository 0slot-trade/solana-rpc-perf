use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;
use env_logger::Builder;
use log::LevelFilter;
use solana_client::rpc_response::SlotUpdate;
use tokio::sync::Mutex;
use std::io::Write;
use chrono::Local;
use clap::Parser;
use futures::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    pub name_0: String,
    #[clap(long)]
    pub websocket_url_0: String,
    
    #[clap(long)]
    pub name_1: String,
    #[clap(long)]
    pub websocket_url_1: String,
}

// Shared state to track slot arrival times
struct SlotTracker {
    slot_times: Mutex<HashMap<String, (String, Instant)>>
}

impl SlotTracker {
    fn new() -> Self {
        Self {
            slot_times: Mutex::new(HashMap::new())
        }
    }

    async fn track_slot(&self, slot: String, endpoint_name: &str) {
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

// WebSocket slot subscription function
async fn websocket_slot_monitor(
    rpc_url: String, 
    endpoint_name: String,
    slot_tracker: Arc<SlotTracker>) -> anyhow::Result<()> {
    let pubsub_client = PubsubClient::new(&rpc_url).await?;

    let (mut slot_stream, _) = pubsub_client.slot_updates_subscribe().await?;
   
    while let Some(slot_update) = slot_stream.next().await {
        match slot_update {
            // SlotUpdate::FirstShredReceived{slot, timestamp: _} => {
            //     slot_tracker.track_slot(format!("{}-first-shread-received", slot), &endpoint_name).await;
            // },
            SlotUpdate::CreatedBank{slot, timestamp: _, parent: _} => {
                slot_tracker.track_slot(format!("{}-create-bank", slot), &endpoint_name).await;
            },
            // SlotUpdate::Completed{slot, timestamp: _} => {
            //     slot_tracker.track_slot(format!("{}-completed", slot), &endpoint_name).await;
            // },
            // SlotUpdate::OptimisticConfirmation{slot, timestamp: _} => {
            //     slot_tracker.track_slot(format!("{}-optimistic-confirmation", slot), &endpoint_name).await;
            // },
            _ => {}
        }
        // slot_tracker.track_slot(slot_update.slot, &endpoint_name).await;
    }
    Ok(())
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

    // Spawn WebSocket monitoring tasks
    let websocket_task_0 = tokio::spawn(websocket_slot_monitor(
        args.websocket_url_0.clone(), 
        args.name_0.clone(),
        slot_tracker.clone()
    ));

    let websocket_task_1 = tokio::spawn(websocket_slot_monitor(
        args.websocket_url_1.clone(), 
        args.name_1.clone(),
        slot_tracker.clone()
    ));

    // Run for desired duration
    tokio::select! {
        result_0 = websocket_task_0 => {
            log::error!("WebSocket 0 task ended: {:?}", result_0);
        },
        result_1 = websocket_task_1 => {
            log::error!("WebSocket 1 task ended: {:?}", result_1);
        },
        _ = tokio::signal::ctrl_c() => {
            log::info!("Received Ctrl+C, shutting down...");
        },
    }

    Ok(())
}

// For Cargo.toml, add these dependencies:
// solana-client = { version = "1.18.0", features = ["full"] }
// solana-sdk = "1.18.0"
// futures = "0.3"