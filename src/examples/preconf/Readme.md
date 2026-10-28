# Shreder Preconf Binary Transactions Example

This example consumes binary Solana transactions received through Shreder's preconf ingest path. Each update contains a serialized Solana [`VersionedTransaction`](https://docs.rs/solana-transaction/latest/solana_transaction/versioned/struct.VersionedTransaction.html), which the client deserializes with `wincode`.

`bincode` cannot read V1 transactions ([SIMD-0385](https://github.com/solana-foundation/solana-improvement-documents/pull/385)): their message comes first and the signatures follow without a length prefix. The example therefore uses `solana-transaction` 6.0 with the `wincode` feature, which reads legacy, V0 and V1:

```toml
solana-transaction = { version = "=6.0.0", features = ["wincode"] }
wincode = "0.6"
```

## Run the example

Set the endpoint in [`main.rs`](./main.rs) to the binary gRPC endpoint provided by the Shreder team, then run:

```bash
cargo run --example preconf
```

The example subscribes to Pump.fun transactions and prints the matching filters, slot, transaction source, transaction version, signature count, and instruction count:

```text
filters: ["pumpfun"], slot: 123456789, preconf: true, version: v1, signatures: 1, instructions: 3
```

## Preconf filter semantics

`SubscribeRequestFilterBinaryTransactions.preconf` selects the transaction source:

- `true` matches only transactions received through the preconf ingest path.
- `false` or an unset value matches only transactions reconstructed from shreds.
- `account_exclude` is not allowed when `preconf` is `true`; this example therefore leaves it empty.

The server reports the actual source in `SubscribeUpdateBinaryTransaction.preconf`.

## Proto setup

The example uses [`proto/shreder_preconf.proto`](../../../proto/shreder_preconf.proto). It retains the `shreder_binary` package and `ShrederBinaryService` names, so the gRPC method path remains compatible with the binary service.

The relevant additions to the binary transaction contract are:

```protobuf
message SubscribeUpdateBinaryTransaction {
  BinaryTransaction transaction = 1;
  uint64 slot = 2;
  bool preconf = 3;
}

message SubscribeRequestFilterBinaryTransactions {
  repeated string account_include = 3;
  repeated string account_exclude = 4;
  repeated string account_required = 6;
  bool preconf = 7;
}
```

Because the repository also keeps the original binary proto with the same protobuf package, `build.rs` generates this schema into a separate `OUT_DIR/preconf` directory. The example includes that generated module with:

```rust
pub mod shreder_binary {
    include!(concat!(env!("OUT_DIR"), "/preconf/shreder_binary.rs"));
}
```

In a standalone client containing only `shreder_preconf.proto`, the usual build script is sufficient:

```rust
use tonic_build::configure;

fn main() {
    const PROTOC_ENVAR: &str = "PROTOC";
    if std::env::var(PROTOC_ENVAR).is_err() {
        #[cfg(not(windows))]
        std::env::set_var(PROTOC_ENVAR, protobuf_src::protoc());
    }

    configure()
        .compile(&["proto/shreder_preconf.proto"], &["proto"])
        .unwrap();
}
```

The module can then use `tonic::include_proto!("shreder_binary")`.

## Subscription

The preconf-only filter used by the example is:

```rust
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
```

As with the other transaction streams, consumers that require exactly-once processing should deduplicate transactions.

Shreder website: https://shreder.xyz/
Discord: https://discord.gg/8qEGZKPVDV
X: https://x.com/ShrederXyz
