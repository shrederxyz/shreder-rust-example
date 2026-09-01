pub mod shredstream {
    tonic::include_proto!("shredstream");
}

use futures::{channel::mpsc::unbounded, sink::SinkExt};
use shredstream::{
    shreder_service_client::ShrederServiceClient, SubscribeRequestFilterTransactions,
    SubscribeTransactionsRequest, SubscribeTransactionsResponse,
};
use tonic::{Response, Streaming};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let entrypoint = "http://fra1.shreder.xyz:9991";
    let mut client = ShrederServiceClient::connect(entrypoint).await.unwrap();

    let request = SubscribeTransactionsRequest {
        transactions: maplit::hashmap! {
            "pumpfun".to_owned() => SubscribeRequestFilterTransactions {
                account_exclude: vec![],
                account_include: vec![],
                account_required: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_owned()]
            }
        },
    };
    let (mut subscribe_tx, subscribe_rx) = unbounded();
    let response: Response<Streaming<SubscribeTransactionsResponse>> =
        client.subscribe_transactions(subscribe_rx).await.unwrap();

    let mut stream = response.into_inner();

    let _ = subscribe_tx.send(request).await;

    while let Some(message) = stream.message().await.unwrap() {
        let update = message.transaction.expect("transaction update must be present");
        let tx = update.transaction.expect("transaction must be present");
        // `Message.config` is optional: set only for V1 (SIMD-0385), absent for legacy/V0.
        let config = tx.message.as_ref().and_then(|m| m.config.clone());

        println!(
            "Filters: {:?}, Sig: {:?}, slot: {}, config: {}",
            message.filters,
            bs58::encode(&tx.signatures[0]).into_string(),
            update.slot,
            match &config {
                Some(c) => format!("{:?}", c),
                None => "none (legacy/v0)".to_owned(),
            },
        );
    }

    Ok(())
}
