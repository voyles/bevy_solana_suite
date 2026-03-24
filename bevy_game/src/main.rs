use bevy::prelude::*;
// Import your macro
use bevy_solana_core::{DeveloperWallet, SolanaClient};
use bevy_solana_gen::generate_bevy_components;
use solana_sdk::signer::Signer; // For accessing pubkey() method on Keypair
use solana_client::rpc_request::RpcRequest;
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};

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
        .add_systems(Startup, (check_bridge, setup_3d_test))
        .add_systems(Update, handle_initialize_input)
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

fn setup_3d_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a Sphere to prove the fragment shader is working on the i9
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 1.0), // Cyan
            reflectance: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Light source
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn handle_initialize_input(
    keys: Res<ButtonInput<KeyCode>>,
    client: Res<SolanaClient>, // This resource has the .client field
    wallet: Res<DeveloperWallet>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let recent_blockhash = client.client.get_latest_blockhash().expect("Failed to get blockhash");
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[initialize()],
            Some(&wallet.keypair.pubkey()),
            &[&wallet.keypair],
            recent_blockhash,
        );
        
        let wire_tx = bincode::serialize(&tx).expect("Failed to serialize");
        let b64_tx = general_purpose::STANDARD.encode(wire_tx);

        match client.client.send::<String>(
            RpcRequest::SendTransaction,
            json!([b64_tx, { "encoding": "base64" }]),
        ) {
            Ok(sig) => info!("🚀 Transaction Sent! Signature: {}", sig),
            Err(e) => error!("❌ RPC Error: {:?}", e),
        }
    }
}
