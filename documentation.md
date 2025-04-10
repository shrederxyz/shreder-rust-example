# Shreder ShredStream Client Implementation Guide

Shreder is the fastest way to get block updates and receive transaction data on Solana.
Behind the scenes, Shreder is a shred streaming service. We stream raw transaction data (shreds) directly to your app — no polling, no delays.

This guide describes the process of creating a client for Shreder ShredStream using Rust and gRPC. Shreder allows you to receive streaming data with Solana network transactions in real-time (0 block).

Shreder website: https://shreder.xyz/
Discord: https://discord.gg/YKFeeVxU

## 1. Setting Up the Project and Dependencies

### a. Create a New Cargo Project

Open your terminal and run the following commands:

```bash
cargo new shredstream_client
cd shredstream_client
```

### b. Edit Cargo.toml

Open the Cargo.toml file and add the required dependencies:

```toml
[package]
name = "shredstream_client"
version = "0.1.0"
edition = "2021"

[dependencies]
tonic = "0.9"                             # Core gRPC library
tokio = { version = "1", features = ["full"] } # Asynchronous runtime
prost = "0.11"                            # Used by Tonic for (de)serialization of messages
bincode = "1.3.3"                         # For deserializing Solana data
solana-entry = "1.16"                     # For working with Solana entries

[build-dependencies]
tonic-build = "0.9"                       # Used to generate Rust code from proto files
```

## 2. Generating Rust Code from the Proto File

### a. Place the Proto File

Create a directory named `proto` in the root of your project, and add the `shredstream.proto` file:

```protobuf
syntax = "proto3";

package shredstream;

service ShredstreamProxy {
  rpc SubscribeEntries(SubscribeEntriesRequest) returns (stream SlotEntries);
}

message SubscribeEntriesRequest {}

message SlotEntries {
  uint64 slot = 1;
  bytes entries = 2;
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

use shredstream::{shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest};
use solana_entry::entry::Entry;
```

### b. Create gRPC Client

In the `main` function, write the following code to create a gRPC connection:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace with your ShredStream server address
    let endpoint = "http://127.0.0.1:9999";
    
    println!("Connecting to ShredStream server at: {}", endpoint);
    
    let mut client = ShredstreamProxyClient::connect(endpoint)
        .await?;
    
    println!("Successfully connected to server");
    
    // Continue with the code...
```

### c. Subscribe to Transaction Updates

Create a stream to get transaction data:

```rust
    println!("Subscribing to entries stream");
    
    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest {})
        .await?
        .into_inner();
    
    println!("Successfully subscribed. Waiting for data...");
```

### d. Handle New Entries

With this stream, you will receive entries. Each Entry contains several transactions.
Type definitions of the entries and transactions can be found in the [documentation](https://docs.rs/solana-entry/latest/solana_entry/entry/struct.Entry.html).

```rust
    while let Some(slot_entry) = stream.message().await? {
        let entries = match bincode::deserialize::<Vec<Entry>>(&slot_entry.entries) {
            Ok(e) => e,
            Err(e) => {
                println!("Deserialization failed with error: {}", e);
                continue;
            }
        };
        
        println!(
            "Slot {}, entries: {}, transactions: {}",
            slot_entry.slot,
            entries.len(),
            entries.iter().map(|e| e.transactions.len()).sum::<usize>()
        );
        
        // Add your custom processing logic here
        process_entries(slot_entry.slot, &entries);
    }
    
    Ok(())
}

fn process_entries(slot: u64, entries: &[Entry]) {
    // Example processing function
    for (i, entry) in entries.iter().enumerate() {
        if !entry.transactions.is_empty() {
            println!("Entry {}: Contains {} transactions", i, entry.transactions.len());
            // Process individual transactions here
        }
    }
}
```

## 4. Complete Example

Here's a complete working example of the ShredStream client:

```rust
pub mod shredstream {
    tonic::include_proto!("shredstream");
}

use shredstream::{shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest};
use solana_entry::entry::Entry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "http://127.0.0.1:9999";
    println!("Connecting to ShredStream server at: {}", endpoint);
    
    let mut client = ShredstreamProxyClient::connect(endpoint)
        .await?;
    
    println!("Successfully connected to server");
    
    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest {})
        .await?
        .into_inner();
    
    println!("Successfully subscribed. Waiting for data...");
    
    while let Some(slot_entry) = stream.message().await? {
        let entries = match bincode::deserialize::<Vec<Entry>>(&slot_entry.entries) {
            Ok(e) => e,
            Err(e) => {
                println!("Deserialization failed with error: {}", e);
                continue;
            }
        };
        
        println!(
            "Slot {}, entries: {}, transactions: {}",
            slot_entry.slot,
            entries.len(),
            entries.iter().map(|e| e.transactions.len()).sum::<usize>()
        );
        
        // Process entries (optional)
        for entry in &entries {
            if !entry.transactions.is_empty() {
                // Custom transaction processing logic
            }
        }
    }
    
    Ok(())
}
```

## 5. Building and Running the Client

To build and run your ShredStream client:

```bash
cargo run
```

Make sure to update the endpoin in this example. You can get access by contacting Shreder team via official Discord or Website.

## 6. Further Resources

- [Solana Documentation](https://docs.solana.com/)
- [Tonic gRPC Documentation](https://github.com/hyperium/tonic)
- [Solana Entry Documentation](https://docs.rs/solana-entry/latest/solana_entry/entry/struct.Entry.html)


Shreder website: https://shreder.xyz/
Discord: https://discord.gg/YKFeeVxU

Connect with us to get access to Shreder.