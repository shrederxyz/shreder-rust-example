# Shreder ShredStream Client Implementation Guide

Shreder is the fastest way to get block updates and receive transaction data on Solana.
Basically, Shreder is a shred streaming service. We stream raw transaction data (shreds) directly to your app — no polling, no delays.

This guide describes the process of creating a client for Shreder ShredStream using Rust and gRPC. Shreder allows you to receive streaming data with Solana network transactions in real-time (0 block).

Shreder website: https://shreder.xyz/
Discord: discord.gg/8qEGZKPVDV
X: https://x.com/ShrederXyz

To run this example please use the following command
```bash
cargo run --example transactions 
```
# Example reconstruction
## 1. Setting Up the Project and Dependencies

### a. Create a New Cargo Project

Open your terminal and run the following commands:

```bash
cargo new shreder_client
cd shreder_client
```

### b. Edit Cargo.toml

Open the Cargo.toml file and add the required dependencies:

```toml
[package]
name = "shreder_client"
version = "0.1.0"
edition = "2025"

[dependencies]
tonic = "0.9"                             # Core gRPC library
tokio = { version = "1", features = ["full"] } # Asynchronous runtime
prost = "0.11"                            # Used by Tonic for (de)serialization of messages
bincode = "1.3.3"                         # For deserializing Solana data
solana-entry = "1.16"                     # For working with Solana entries
futures = "0.3.24"
maplit = "1.0.2"
prost-types = "0.12"
bs58 = "0.4"

[build-dependencies]
tonic-build = "0.9"                       # Used to generate Rust code from proto files
protobuf-src = "1"
```

## 2. Generating Rust Code from the Proto File

### a. Place the Proto File

Create a directory named `proto` in the root of your project, and add the `shredstream.proto` file:

```protobuf
syntax = "proto3";

package shredstream;
import "google/protobuf/timestamp.proto";



service ShrederService {
  rpc SubscribeEntries(SubscribeEntriesRequest) returns (stream Entry);
  rpc SubscribeTransactions(stream SubscribeTransactionsRequest) returns (stream SubscribeTransactionsResponse);
}

message SubscribeEntriesRequest {
  // tbd: we may want to add filters here
}

message SubscribeTransactionsRequest {
  map<string, SubscribeRequestFilterTransactions> transactions = 3;
}

message SubscribeTransactionsResponse {
  repeated string filters = 1;
  SubscribeUpdateTransaction transaction = 4;
  google.protobuf.Timestamp created_at = 11;
}

message SubscribeUpdateTransaction {
  Transaction transaction = 1;
  uint64 slot = 2;
}

message SubscribeRequestFilterTransactions {
  repeated string account_include = 3;
  repeated string account_exclude = 4;
  repeated string account_required = 6;
}

message Entry {
  // the slot that the entry is from
  uint64 slot = 1;

  // Serialized bytes of Vec<Entry>: https://docs.rs/solana-entry/latest/solana_entry/entry/struct.Entry.html
  bytes entries = 2;
}

message MessageHeader {
  uint32 num_required_signatures = 1;
  uint32 num_readonly_signed_accounts = 2;
  uint32 num_readonly_unsigned_accounts = 3;
}

message CompiledInstruction {
  uint32 program_id_index = 1;
  bytes accounts = 2;
  bytes data = 3;
}

message MessageAddressTableLookup {
  bytes account_key = 1;
  bytes writable_indexes = 2;
  bytes readonly_indexes = 3;
}

message Message {
  MessageHeader header = 1;
  repeated bytes account_keys = 2;
  bytes recent_blockhash = 3;
  repeated CompiledInstruction instructions = 4;
  bool versioned = 5;
  repeated MessageAddressTableLookup address_table_lookups = 6;
}

message Transaction {
  repeated bytes signatures = 1;
  Message message = 2;
}
```

### b. Create the build.rs File

In the root of your project, create a file named `build.rs` with the following content to compile the proto file:

```rust
fn main() {
    tonic_build::compile_protos("proto/shredstream.proto")
        .expect("Failed to compile proto files");
}
```

When you build the project, Cargo will execute `build.rs` to generate the Rust modules from your proto definitions.

## 3. Implementing the gRPC Client

### a. Import Dependencies

Add dependencies to connect to the ShredStream server:

```rust
// Import the generated modules. The string here should match the package name in your proto.
pub mod shredstream {
    tonic::include_proto!("shredstream");
}

use futures::{channel::mpsc::unbounded, sink::SinkExt};
use shredstream::{
    shreder_service_client::ShrederServiceClient, SubscribeRequestFilterTransactions,
    SubscribeTransactionsRequest, SubscribeTransactionsResponse,
};
use tonic::{Response, Streaming};
```

### b. Create gRPC Client

In the `main` function, write the following code to create a gRPC connection:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace with your ShredStream server address
    let endpoint = "http://127.0.0.1:9999";
    
    println!("Connecting to ShredStream server at: {}", endpoint);
    
    let mut client = ShrederServiceClient::connect(endpoint)
        .await?;
    
    println!("Successfully connected to server");
    
    // Continue with the code...
```

### c. Subscribe to Transaction Updates

Create a subscribe transaction request with the following format:
``` rust
    let request = SubscribeTransactionsRequest {
        transactions: maplit::hashmap! {
            "pumpfun".to_owned() => SubscribeRequestFilterTransactions {
                // excludes transactions involving any of these accounts
                account_exclude: vec![],
                // allows only transactions involving any of these accounts
                account_include: vec![],
                // allows only transactions involving all of these accounts
                account_required: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_owned()]
            }
        },
    };
```
Create a stream and send incoming channel to the server. As a response you will get stream object that you can be used for obtaining new transactions:
```rust
    let (mut subscribe_tx, subscribe_rx) = unbounded();
    let response: Response<Streaming<SubscribeTransactionsResponse>> =
        client.subscribe_transactions(subscribe_rx).await.unwrap();

    let mut stream = response.into_inner();
```

Send the filter request into the outgoing channel
```rust
    let _ = subscribe_tx.send(request).await;
```

### d. Handle New Entries

With this stream, you will receive transactions based on your filter. Each transaction follows the format described in the [proto file](/proto/shredstream.proto).\
**NOTE:** A transaction may be sent multiple times. Please ensure on your end that you process each transaction only once.
```rust
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
```

## 4. Building and Running the Client

To build and run your ShredStream client:

```bash
cargo run
```

Make sure to update the endpoin in this example. You can get access by contacting Shreder team via official Discord or Website.

## 5. Further Resources

- [Solana Documentation](https://docs.solana.com/)
- [Tonic gRPC Documentation](https://github.com/hyperium/tonic)
- [Solana Entry Documentation](https://docs.rs/solana-entry/latest/solana_entry/entry/struct.Entry.html)


Shreder website: https://shreder.xyz/
X: https://x.com/ShrederXyz

Connect with us to get access to Shreder.
