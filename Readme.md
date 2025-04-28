# Shreder Rust Example

Shreder is the fastest way to get block updates and receive transaction data on Solana.
Basically, Shreder is a shred streaming service. We stream raw transaction data (shreds) directly to your app — no polling, no delays.
Shreder allows you to receive streaming data with Solana network transactions in real-time (0 block).

## Description

This repository demonstrates a basic project to test Shreder and consume transactions. It contains 2 examples with entries and transactions consuming.

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

    In the src/examples/entries/main.rs or src/examples/transactions/main.rs file change http://localhost:9991 to url link provided by the Shreder team. Connect with us via official website or Discord.

3. **Run script**

   To run [entries](/src/examples/entries/) example
   ```bash
      cargo run --example entries
   ```
   For additional information about entries use [documentation](/src/examples/entries/Readme.md)

   To run [transactions](/src/examples/transactions/) example
   ```bash
      cargo run --example transactions
   ```
   For additional information about transactions use [documentation](/src/examples/transactions/Readme.md)

Shreder website: https://shreder.xyz/
Discord: https://discord.gg/8qEGZKPVDV
X: https://x.com/ShrederXyz
