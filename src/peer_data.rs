
// Std imports
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Result as IOResult};
use std::str::FromStr;
// External imports
use libp2p::{PeerId, Swarm};
use libp2p::identity::{Keypair, PublicKey};
use libp2p::websocket::tls::PrivateKey;
use libp2p::identity::ed25519::SecretKey;
use libp2p::gossipsub::Gossipsub;
use libp2p::swarm::NetworkBehaviour;
use libp2p::core::Multiaddr;
use serde::{Serialize, Deserialize};

// Private key file path
const KEY_FILE: &'static str = "./id_dsa";
// Public key file path
const PUB_KEY_FILE: &'static str = "./id_dsa.pub";

// Buffer size for reading DER for signing
const MAX_DER_SIZE: usize = 2048;

// File for storing known peers
const KNOWN_PEERS_PATH: &'static str = "./peer_ids.json";

// How many bytes to expect to be stored in known peers file
const MAX_PEER_FILE_SIZE: usize = 4096;

// Data needed to connect to a previously known peer (we also can store this for ourself)
#[derive(Serialize, Deserialize, Debug)]
pub struct PeerData {
    id: String,
    ip: String,
}

impl PeerData {
    pub fn peer_id(&self) -> PeerId {
        PeerId::from_str(self.id.as_str()).expect("Failed to parse peer id")
    }
    pub fn ip(&self) -> Multiaddr {
        self.ip.parse().expect("Failed to parse peer multi address")
    }

    pub fn new(peer_id: &PeerId, swarm: &mut Swarm<Gossipsub>) -> Self {
        let multi_address = swarm.addresses_of_peer(&peer_id).first().expect("Peer found without IP").to_string();
        Self {
            id: peer_id.to_string(),
            ip: multi_address,
        }
    }
}

// pub fn my_peer_data() -> PeerData {
//     let peer_id = get_peer_id(get_keypair());
//
//     PeerData {
//         id: peer_id.to_string(),
//         ip: todo!("get external ip"),
//     }
// }

pub fn get_keypair() -> Keypair {
    if let Ok(mut private_key_file) = File::open(Path::new(KEY_FILE)) {
        if let Ok(public_key_file) = File::open(Path::new(PUB_KEY_FILE)) {
            let mut der_buffer = [0u8; MAX_DER_SIZE];
            private_key_file.read(&mut der_buffer);
            return Keypair::secp256k1_from_der(&mut der_buffer).expect("Failed to parse private key file")
        }
    }

    let keypair = Keypair::generate_secp256k1();
    // TODO: Save the new keys to a file for later use
    // save_keys(keypair.clone());

    keypair
}

fn save_keys(keypair: Keypair) -> IOResult<()> {
    match keypair {
        Keypair::Secp256k1(keypair) => {
            // Get key byte values
            let public_key = keypair.public().encode();
            let private_key = keypair.secret().to_bytes();

            // Write private key to file
            let mut private_key_file = File::create(Path::new(KEY_FILE))?;
            private_key_file.write(&private_key);
            // Write public key to file
            let mut public_key_file = File::create(Path::new(PUB_KEY_FILE))?;
            public_key_file.write(&public_key);
        },
        _ => panic!("unsupported signing algorithm"),
    };

    Ok(())
}

pub fn save_known_peer(peer_data: PeerData) {
    // Open the file in r/w mode (will create if non-existent)
    let mut file = OpenOptions::new().write(true).create(true).read(true).open(Path::new(KNOWN_PEERS_PATH)).expect("Failed to open known peers file");
    let mut contents = String::with_capacity(MAX_PEER_FILE_SIZE);
    file.read_to_string(&mut contents);
    let mut known_peers: Vec<PeerData> = serde_json::from_str(contents.as_str()).unwrap_or_default();
    known_peers.push(peer_data);
    file.write_all(&serde_json::to_vec(&known_peers).expect("Failed to serialize known peers to JSON")).expect("Failed to write to known peers file");
}

pub fn save_known_peers(peers_data: Vec<PeerData>) {
    // Open the file in write mode (will create if non-existent)
    let mut file = OpenOptions::new().write(true).create(true).open(KNOWN_PEERS_PATH).expect("Failed to open known peers file");
    file.write_all(&serde_json::to_vec(&peers_data).expect("Failed to serialize known peers to JSON")).expect("Failed to write to known peers file");
}

pub fn get_known_peers() -> Vec<PeerData> {
    if let Ok(mut peers_file) = File::open(KNOWN_PEERS_PATH) {
        let mut contents = String::with_capacity(MAX_PEER_FILE_SIZE);
        peers_file.read_to_string(&mut contents);
        serde_json::from_slice(contents.as_bytes()).expect("Failed to parse peer data in peers file")
        // TODO: Reset peers file and return empty vec if we fail to parse file
    } else {
        vec![]
    }
}
