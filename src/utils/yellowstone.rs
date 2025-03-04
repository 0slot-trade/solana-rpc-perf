use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeRequestFilterSlots};
use yellowstone_grpc_proto::prelude::subscribe_update::UpdateOneof;
use yellowstone_grpc_proto::prelude::SubscribeRequest;
use backoff::future::retry;
use backoff::backoff::Constant;

pub async fn create_yellowstone_client(endpoint: &String,
    x_token: &Option<String>, timeout_secs: u64) -> anyhow::Result<GeyserGrpcClient<impl Interceptor>> {
    GeyserGrpcClient::build_from_shared(endpoint.clone())
                .and_then(|builder| builder.x_token(x_token.clone()))
                .map(|builder| builder.connect_timeout(Duration::from_secs(timeout_secs)))
                .map(|builder| builder.timeout(Duration::from_secs(timeout_secs)))
                .map_err(|e| anyhow::Error::msg(format!("Failed to create builder: {}", e)))?
                .connect()
                .await
                .map_err(|e| {
                    anyhow::Error::msg(format!(
                        "Failed to connected to endpoint: {} ({})",
                        endpoint, e
                    ))
                })
}


pub async fn slot_subscribe_to_channel(
    endpoint: String,
    x_token: Option<String>,
    slot_channel: Arc<Sender<u64>>,
    commitment: CommitmentLevel,
) -> anyhow::Result<()> {
    let mut slots = HashMap::new();
    slots.insert(
        "client".to_string(),
        SubscribeRequestFilterSlots { filter_by_commitment: Some(true) },
    );

    // The default exponential backoff strategy intervals:
    // [500ms, 750ms, 1.125s, 1.6875s, 2.53125s, 3.796875s, 5.6953125s,
    // 8.5s, 12.8s, 19.2s, 28.8s, 43.2s, 64.8s, 97s, ... ]
    retry(Constant::new(Duration::from_secs(1)), move || {
        let (endpoint, x_token) = (endpoint.clone(), x_token.clone());
        let slot_channel = slot_channel.clone();
        let slots = slots.clone();
        async move {
            log::info!("Reconnecting to the slot gRPC server");
            let mut client = create_yellowstone_client(&endpoint, &x_token, 10).await?;
            let subscribe_request = SubscribeRequest {
                slots,
                accounts: HashMap::new(),
                transactions: HashMap::new(),
                blocks: HashMap::new(),
                blocks_meta: HashMap::new(),
                commitment: Some(commitment.into()), // 1 是 confirmed
                accounts_data_slice: vec![],
                transactions_status: HashMap::new(),
                ping: None,
                entry: HashMap::new(),
            };
            let (mut _subscribe_tx, mut stream) = client
                .subscribe_with_request(Some(subscribe_request))
                .await
                .map_err(|e| anyhow::Error::msg(format!("Failed to subscribe: {}", e)))?;
            
            while let Some(message) = stream.next().await {
                if message.is_err() {
                    log::error!("Failed to receive slot update, break the stream loop and retry");
                    break;
                }
                let msg = message.unwrap();
                match msg.update_oneof {
                    Some(UpdateOneof::Slot(slot)) => {
                        slot_channel.send(slot.slot).await.unwrap();
                    },
                    Some(UpdateOneof::Ping(_ping)) => {
                        // log::info!("Received slot subscribe ping: {:?}", ping);
                        // let r = subscribe_tx
                        //     .send(SubscribeRequest {
                        //         ping: Some(SubscribeRequestPing { id: 1 }),
                        //         ..Default::default()
                        //     })
                        //     .await;
                        // if let Err(e) = r {
                        //     log::error!("Failed to send ping: {}", e);
                        //     continue;
                        // }
                    },
                    Some(UpdateOneof::Pong(_pong)) => {
                        // log::info!("Received slot subscribe pong: {:?}", pong);
                    }
                    None => {
                        log::info!("received none update one of");
                        break;
                    },
                    _ => {
                    }
                }

            }
            Err(anyhow::Error::msg("Stream closed").into())
        }
    })
    .await
    .map_err(Into::into)
}
