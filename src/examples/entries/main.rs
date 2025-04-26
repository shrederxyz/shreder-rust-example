pub mod shredstream {
    tonic::include_proto!("shredstream");
}

use shredstream::{shreder_service_client::ShrederServiceClient, SubscribeEntriesRequest};
use tonic::{Response, Status, Streaming};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let entrypoint = "http://localhost:9991";
    let mut client = ShrederServiceClient::connect(entrypoint).await.unwrap();
    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest {})
        .await
        .unwrap()
        .into_inner();

    while let Some(slot_entry) = stream.message().await.unwrap() {
        let entries =
            match bincode::deserialize::<Vec<solana_entry::entry::Entry>>(&slot_entry.entries) {
                Ok(e) => e,
                Err(e) => {
                    println!("Deserialization failed with err: {e}");
                    continue;
                }
            };

        println!(
            "slot {}, entries: {}, transactions: {}",
            slot_entry.slot,
            entries.len(),
            entries.iter().map(|e| e.transactions.len()).sum::<usize>()
        );
    }
    Ok(())
}
