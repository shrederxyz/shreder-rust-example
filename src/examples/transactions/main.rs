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
    let entrypoint = "http://localhost:9991";
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
        println!(
            "Filters: {:?}, Sig: {:?}",
            message.filters,
            bs58::encode(&message.transaction.unwrap().transaction.unwrap().signatures[0])
                .into_string()
        )
    }

    Ok(())
}
