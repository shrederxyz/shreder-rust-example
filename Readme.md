# Shreder Rust Example

Shreder is the fastest way to get block updates and receive transaction data on Solana.
Behind the scenes, Shreder is a shred streaming service. We stream raw transaction data (shreds) directly to your app — no polling, no delays.
Shreder allows you to receive streaming data with Solana network transactions in real-time (0 block).

## Description

This repository demonstrates a basic project to test Shreder and consume transactions.

## Getting Started

Follow these instructions to set up the project on your local machine.

### Prerequisites

Before you begin, make sure you have the following installed:

- [Git](https://git-scm.com/) for cloning the repository
- [Rust](https://www.rust-lang.org/tools/install) for building the project

### Usage
1. **Clone the repository**
  ```bash
   git clone https://github.com/shrederxyz/shreder-rust-example.git
   ```

2.  **Configure the project**

    In the src/main.rs file change http://127.0.0.1:9999 to url link provided by the Shreder team. Connect with us via official website or Discord.

3. **Run script**

```bash
   cargo run
```
For additional information use [documentation](/documentation.md)

Shreder website: https://shreder.xyz/
Discord: https://discord.gg/YKFeeVxU
