use bevy::prelude::*;
// Import your macro
use bevy_solana_gen::generate_bevy_components;
use bevy_solana_core::{SolanaClient, DeveloperWallet};
use solana_sdk::signer::Signer; // For accessing pubkey() method on Keypair

// This triggers the file-reading and code-generation
// The path is relative to the 'bevy_game' folder
generate_bevy_components!("../anchor_test_program/target/idl/anchor_test_program.json");

fn main() {
    let home = std::env::var("HOME").expect("HOME not set");
    let wallet_path = format!("{}/.config/solana/id.json", home);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SolanaClient::new("http://127.0.0.1:8899"))
        .insert_resource(DeveloperWallet::new_from_file(&wallet_path))
        .add_systems(Startup, check_bridge)
        .run();
}

// Add 'wallet: Res<DeveloperWallet>' to the arguments
fn check_bridge(solana: Res<SolanaClient>, wallet: Res<DeveloperWallet>) {
    match solana.client.get_slot() {
        Ok(slot) => {
            // Get the public key from the loaded keypair
            let pubkey = wallet.keypair.pubkey();
            
            info!("Solana-Bevy Bridge is LIVE.");
            info!("Current Slot: {}", slot);
            info!("Wallet Address: {}", pubkey);
            
            let _call = Initialize {};
            info!("Macro-generated 'Initialize' is ready.");
        }
        Err(e) => {
            error!("Bridge failed to reach validator: {}", e);
        }
    }
}
