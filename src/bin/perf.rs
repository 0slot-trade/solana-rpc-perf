use std::sync::Arc;
use env_logger::Builder;
use log::LevelFilter;
use solana_rpc_perf::utils::yellowstone::slot_subscribe_to_channel;
use tokio::sync::mpsc::{self, Receiver};
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

fn log_slot(name: &str, slot: u64)  {
    log::info!("Logging slot {} for endpoint {}", slot, name);
}

async fn slot_monitor(
    endpoint_name: String,
    mut receiver: Receiver<u64>,
) {
    while let Some(slot) = receiver.recv().await {
        log_slot(&endpoint_name, slot);
    }
}

fn subscribe(
    endpoint_name: String,
    grpc_url: String,
    x_token: Option<String>,
) -> (tokio::task::JoinHandle<()>, impl std::future::Future<Output = anyhow::Result<()>>) {
    let (tx, rx) = mpsc::channel(1000);
    let monitor = tokio::spawn(slot_monitor(
        endpoint_name.clone(),
        rx,
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


    let (monitor0, sub0) = subscribe(
        args.name_0.clone(),
        args.grpc_url_0.clone(),
        args.x_token_0.clone(),
    );
    let (monitor1, sub1) = subscribe(
        args.name_1.clone(),
        args.grpc_url_1.clone(),
        args.x_token_1.clone(),
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