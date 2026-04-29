# Shreder ShredStream Client Implementation Guide (Binary transactions)

Shreder is the fastest way to get block updates and receive transaction data on Solana.
Basically, Shreder is a shred streaming service. We stream raw transaction data (shreds) directly to your app — no polling, no delays.

This guide describes how to consume the **binary transactions** gRPC API: each update includes a `bytes` payload that is a serialized Solana [`VersionedTransaction`](https://docs.rs/solana-transaction/latest/solana_transaction/versioned/struct.VersionedTransaction.html). You deserialize it in Rust (this example uses `bincode`) and process instructions, accounts, etc. locally.

Shreder website: https://shreder.xyz/
Discord: discord.gg/8qEGZKPVDV
X: https://x.com/ShrederXyz

To run this example please use the following command

```bash
cargo run --example binary
```

# Example reconstruction

## 1. Setting Up the Project and Dependencies

### a. Create a New Cargo Project

Open your terminal and run the following commands:

```bash
cargo new shreder_binary_client
cd shreder_binary_client
```

### b. Edit Cargo.toml

Open the `Cargo.toml` file and add the required dependencies. Versions match the root [`Cargo.toml`](../../../Cargo.toml) of this repository (`shreder-client-example`). For a crate that only implements the binary client, you may omit `solana-entry` and `bs58` (they are used by the other examples).

```toml
[package]
name = "shreder_binary_client"
version = "0.1.0"
edition = "2021"

[dependencies]
bincode = "1.3.3"
futures = "0.3.24"
solana-entry = "=2.2.1"
solana-transaction = "=2.2.1"
tokio = { version = "1", features = ["full"] }
tonic = { version = "0.10", features = ["tls", "tls-roots", "tls-webpki-roots"] }
maplit = "1.0.2"
prost = "0.12"
prost-types = "0.12"
protobuf-src = "1"
bs58 = "0.4"

[build-dependencies]
protobuf-src = "1"
tonic-build = "0.10"
```

## 2. Generating Rust Code from the Proto File

### a. Place the Proto File

Create a directory named `proto` in the root of your project and add [`shreder_binary.proto`](/proto/shreder_binary.proto). Minimal contents:

```protobuf
syntax = "proto3";

package shreder_binary;

import "google/protobuf/timestamp.proto";

service ShrederBinaryService {
  rpc SubscribeBinaryTransactions(stream SubscribeBinaryTransactionsRequest) returns (stream SubscribeBinaryTransactionsResponse);
}

message SubscribeBinaryTransactionsRequest {
  map<string, SubscribeRequestFilterBinaryTransactions> transactions = 3;
}

message SubscribeBinaryTransactionsResponse {
  repeated string filters = 1;
  SubscribeUpdateBinaryTransaction transaction = 4;
  google.protobuf.Timestamp created_at = 11;
}

message SubscribeUpdateBinaryTransaction {
  BinaryTransaction transaction = 1;
  uint64 slot = 2;
}

message SubscribeRequestFilterBinaryTransactions {
  repeated string account_include = 3;
  repeated string account_exclude = 4;
  repeated string account_required = 6;
}

message BinaryTransaction {
  repeated bytes signatures = 1;
  // Serialized bytes of VersionedTransaction
  bytes binary_transaction = 3;
}
```

You also need the well-known protobuf types (e.g. `google/protobuf/timestamp.proto`). The `tonic-build` / `protoc` setup in `build.rs` below uses an include path so imports resolve.

### b. Create the `build.rs` File

In the root of your project, create `build.rs` so Cargo generates Rust modules from your proto definitions:

```rust
use tonic_build::configure;

fn main() {
    const PROTOC_ENVAR: &str = "PROTOC";
    if std::env::var(PROTOC_ENVAR).is_err() {
        #[cfg(not(windows))]
        std::env::set_var(PROTOC_ENVAR, protobuf_src::protoc());
    }

    configure()
        .compile(&["proto/shreder_binary.proto"], &["proto"])
        .unwrap();
}
```

When you build the project, Cargo runs `build.rs` and emits modules under `OUT_DIR`; `tonic::include_proto!("shreder_binary")` loads the generated code (the string must match `package shreder_binary` in the `.proto`).

## 3. Implementing the gRPC Client

### a. Import Dependencies

```rust
pub mod shreder_binary {
    tonic::include_proto!("shreder_binary");
}

use futures::{channel::mpsc::unbounded, sink::SinkExt};
use shreder_binary::{
    shreder_binary_service_client::ShrederBinaryServiceClient, SubscribeBinaryTransactionsRequest,
    SubscribeBinaryTransactionsResponse, SubscribeRequestFilterBinaryTransactions,
};
use solana_transaction::versioned::VersionedTransaction;
use tonic::{Response, Streaming};
```

### b. Create gRPC Client

```rust
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let entrypoint = "http://fra.binary.shreder.xyz:9991";

    let mut client = ShrederBinaryServiceClient::connect(entrypoint).await.unwrap();

    // ...
```

Use the binary gRPC endpoint you were given; the value above matches [`src/examples/binary/main.rs`](./main.rs) in this repo and may differ from the classic ShredStream host.

### c. Subscribe to binary transaction updates

`SubscribeBinaryTransactions` is a **client-streaming** RPC: you send one or more `SubscribeBinaryTransactionsRequest` messages on a channel; the server responds with a stream of `SubscribeBinaryTransactionsResponse`.

Filters mirror the transactions API: `account_include`, `account_exclude`, `account_required`.

```rust
    let request = SubscribeBinaryTransactionsRequest {
        transactions: maplit::hashmap! {
            "pumpfun".to_owned() => SubscribeRequestFilterBinaryTransactions {
                account_exclude: vec![],
                account_include: vec![],
                account_required: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_owned()],
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
```

### d. Handle responses and deserialize `VersionedTransaction`

Each response may carry `transaction` with nested `BinaryTransaction`: use `binary_transaction` (bytes) as the serialized `VersionedTransaction`, and `signatures` as provided by the API.

**NOTE:** As with other streams, a logical transaction may appear more than once—deduplicate if your use case requires exactly-once processing.

```rust
    while let Some(response) = stream.message().await.unwrap() {
        let update = response.transaction.expect("transaction must be present");
        let tx = update.transaction.expect("transaction must be present");

        let versioned_tx = match bincode::deserialize::<VersionedTransaction>(&tx.binary_transaction) {
            Ok(vt) => vt,
            Err(e) => {
                println!("Failed to deserialize VersionedTransaction: {e}");
                continue;
            }
        };

        let instruction_count = versioned_tx.message.instructions().len();
        println!(
            "filters: {:?}, slot {}, signatures: {}, instructions: {}",
            response.filters,
            update.slot,
            tx.signatures.len(),
            instruction_count,
        );
    }

    Ok(())
}
```

## 4. Complete Example

A complete working example matching this repository lives in [`main.rs`](./main.rs).

## 5. Building and Running the Client

To build and run from this repository:

```bash
cargo run --example binary
```

Make sure to update the endpoint in the example. You can get access by contacting the Shreder team via the official Discord, Telegram or website.

## 6. Further Resources

- [Solana Documentation](https://docs.solana.com/)
- [Tonic gRPC Documentation](https://github.com/hyperium/tonic)
- [VersionedTransaction (solana-transaction)](https://docs.rs/solana-transaction/latest/solana_transaction/versioned/struct.VersionedTransaction.html)

Shreder website: https://shreder.xyz/
Discord: discord.gg/8qEGZKPVDV
X: https://x.com/ShrederXyz

Connect with us to get access to Shreder.
