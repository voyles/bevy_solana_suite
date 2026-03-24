use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::EncodableKey;

#[derive(Resource)]
pub struct SolanaClient {
    pub client: RpcClient,
}
impl SolanaClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: RpcClient::new(url.to_string()),
        }
    }
}

#[derive(Resource)]
pub struct DeveloperWallet {
    pub keypair: Keypair,
}

impl DeveloperWallet {
    pub fn new_from_file(path: &str) -> Self {
        let keypair = Keypair::read_from_file(path).expect("Failed to read developer keypair. Run `solana-keygen new' if it doesn't exist.");
        Self { keypair }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct WrappedPubkey(pub [u8; 32]);

impl WrappedPubkey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}