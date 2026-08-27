pub mod shreder_binary {
    include!(concat!(env!("OUT_DIR"), "/preconf/shreder_binary.rs"));
}

use futures::{channel::mpsc::unbounded, sink::SinkExt};
use shreder_binary::{
    shreder_binary_service_client::ShrederBinaryServiceClient, SubscribeBinaryTransactionsRequest,
    SubscribeBinaryTransactionsResponse, SubscribeRequestFilterBinaryTransactions,
};
use solana_transaction::versioned::VersionedTransaction;
use tonic::{Response, Streaming};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let entrypoint = "http://fra-allnodes.binary.shreder.xyz:9991";
    let mut client = ShrederBinaryServiceClient::connect(entrypoint)
        .await
        .unwrap();

    let request = SubscribeBinaryTransactionsRequest {
        transactions: maplit::hashmap! {
            "pumpfun".to_owned() => SubscribeRequestFilterBinaryTransactions {
                account_include: vec![],
                account_exclude: vec![],
                account_required: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_owned()],
                preconf: true,
            }
        },
    };

    let (mut subscribe_tx, subscribe_rx) = unbounded();
    let response: Response<Streaming<SubscribeBinaryTransactionsResponse>> = client
        .subscribe_binary_transactions(subscribe_rx)
        .await
        .unwrap();
    let mut stream = response.into_inner();

    let _ = subscribe_tx.send(request).await;

    while let Some(response) = stream.message().await.unwrap() {
        let update = response.transaction.expect("transaction must be present");
        let tx = update.transaction.expect("transaction must be present");

        let versioned_tx =
            match bincode::deserialize::<VersionedTransaction>(&tx.binary_transaction) {
                Ok(vt) => vt,
                Err(e) => {
                    println!("Failed to deserialize VersionedTransaction: {e}");
                    continue;
                }
            };

        let instruction_count = versioned_tx.message.instructions().len();
        println!(
            "filters: {:?}, slot: {}, preconf: {}, signatures: {}, instructions: {}",
            response.filters,
            update.slot,
            update.preconf,
            tx.signatures.len(),
            instruction_count,
        );
    }

    Ok(())
}
