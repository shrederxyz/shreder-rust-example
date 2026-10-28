pub mod shredstream {
    tonic::include_proto!("shredstream");
}

use shredstream::{shreder_service_client::ShrederServiceClient, SubscribeEntriesRequest};
use solana_hash::Hash;
use solana_transaction::versioned::VersionedTransaction;
use wincode::{containers, len::BincodeLen, SchemaRead};

// Same limit as `solana_entry::entry::MaxDataShredsLen`: max data shreds per slot * packet size.
const MAX_DATA_SHREDS_SIZE: usize = 32_768 * 1232;

/// Wire-compatible copy of `solana_entry::entry::Entry`. Entries are wincode-encoded,
/// which also covers V1 transactions (SIMD-0385); bincode cannot read those.
#[derive(Debug, SchemaRead)]
pub struct Entry {
    pub num_hashes: u64,
    pub hash: Hash,
    #[wincode(with = "containers::Vec<VersionedTransaction, BincodeLen<MAX_DATA_SHREDS_SIZE>>")]
    pub transactions: Vec<VersionedTransaction>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let entrypoint = "http://fra1.shreder.xyz:9991";
    let mut client = ShrederServiceClient::connect(entrypoint).await.unwrap();
    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest {})
        .await
        .unwrap()
        .into_inner();

    while let Some(slot_entry) = stream.message().await.unwrap() {
        let entries = match wincode::deserialize::<Vec<Entry>>(&slot_entry.entries) {
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
