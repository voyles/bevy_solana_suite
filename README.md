# Bevy Solana Suite 🦀🪐

A high-performance Rust workspace integrating the **Bevy Game Engine (v0.15)** with the **Solana Blockchain (Agave/SDK v4.0)**. 

This suite enables real-time blockchain state synchronization and transaction signing directly within a Bevy ECS (Entity Component System) environment.

## 🏗 Project Architecture

The workspace is split into three core crates to maintain a clean separation of concerns:

| Crate | Responsibility |
| :--- | :--- |
| **`bevy_solana_gen`** | Procedural Macro that parses **Anchor IDLs** and generates Bevy-compatible Components and Structs. |
| **`bevy_solana_core`** | Centralized Resource management for the `SolanaClient` (RPC) and `DeveloperWallet` (Keypair management). |
| **`bevy_game`** | The actual game binary. Implements systems that interact with both the GPU and the Solana Validator. |

## 🚀 Quick Start (Development)

### 1. Start the Local Validator
&nbsp;&nbsp; In a separate terminal, launch a fresh Solana ledger:
```bash
solana-test-validator --reset
```

### 2. Run the Bevy App
&nbsp;&nbsp;From the root directory:
```bash
cargo run -p bevy_game
```

## 🛠 Tech Stack
&nbsp;&nbsp;Engine: [Bevy](https://bevyengine.org/)

&nbsp;&nbsp;Blockchain: [Solana Agave SDK](https://github.com/anza-xyz/agave)

&nbsp;&nbsp;Environment: Kali Linux via WSL2 / Windows 11

&nbsp;&nbsp;Hardware: Intel i9-13900K

## 📝 Current Status (v0.1.0)
&nbsp;&nbsp;[x] Workspace & Crate linking.

&nbsp;&nbsp;[x] Procedural Macro IDL Parsing.

&nbsp;&nbsp;[x] RPC Client "Handshake" with local validator.

&nbsp;&nbsp;[x] Filesystem-based Developer Wallet loading.

&nbsp;&nbsp;[x] Instruction Discriminator helper.

&nbsp;&nbsp;[x] First Initialize transaction execution.

&nbsp;&nbsp;[x] Rich IDL Type Support

&nbsp;&nbsp;[x] Account Deserializers

&nbsp;&nbsp;[&nbsp;] The Marker Component

&nbsp;&nbsp;[&nbsp;] The Signer System

&nbsp;&nbsp;[&nbsp;] The Submission System



