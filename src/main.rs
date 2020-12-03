
// Modules
mod swarm;
mod peer_data;

// Local imports
use crate::swarm::{spawn_swarm, BLOCKCHAIN_TOPIC, IDENTIFY_TOPIC, dial_address};
use crate::peer_data::{get_keypair, get_known_peers, save_known_peer, PeerData, save_known_peers};
// Std imports
use std::str;
use std::env;
use std::{
    error::Error,
    task::{Context, Poll},
};
// External imports
use futures::prelude::*;
use async_std::task;
use libp2p::gossipsub::{GossipsubEvent, Topic};
use serde::{Serialize, Deserialize};
use futures::StreamExt;
use libp2p::swarm::NetworkBehaviour;
use libp2p::core::Multiaddr;
use libp2p::PeerId;

const DEFAULT_PORT: u16 = 4000;
const MAX_PEERS: usize = 10;

// TEMP: Blockchain data
type Block = i32;
#[derive(Serialize, Deserialize, Debug)]
struct BlockChainDummy {
    block_chain: Vec<Block>,
    difficulty: i32,
}

fn process_data_stream(msg: &str, _id: String, _peer_id: String) {
    println!("This is the message: {}", msg);
    // let p = env::current_dir().unwrap();
    // let temp = p.to_string_lossy();
    // let mut path = temp.to_string();
    // let bar = "/src/peer_ids.json".to_string();
    // path.push_str(&bar);
    //
    // let path_present = std::path::Path::new(&path).exists();
    //
    // //fs::write(path, msg).expect("Unable to write file");
    // if path_present{
    //     let mut file = OpenOptions::new().append(true).open(path).expect("File open failed");
    //     file.write_all(msg.as_bytes()).expect("write failed");
    // }
    // else{
    //     let mut f = File::create(path).expect("Unable to create file");
    //     f.write_all(msg.as_bytes()).expect("Unable to write data");
    // }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Blockchain CS5600");
    // Get our peer data to start swarm
    let my_keypair = get_keypair();
    let my_peer_id = PeerId::from_public_key(my_keypair.public());

    println!("Me {}", my_peer_id.to_string());

    // Create a Swarm to manage peers and events
    let mut swarm = spawn_swarm(my_keypair, my_peer_id.clone());

    println!("Spawned Swarm");

    // Listen on all interfaces and whatever port the OS assigns
    libp2p::Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().expect("Invalid swarm ip")).expect("Failed to start swarm listen");

    println!("Swarm Listening");

    // Reach out to another node if specified
    if let Some(to_dial) = std::env::args().nth(1) {
        match to_dial.parse() {
            Ok(address) => dial_address(address, &mut swarm),
            Err(err) => eprintln!("Failed to parse address to dial: {:?}", err),
        }
    }

    // Get all previously known peers
    let known_peers = get_known_peers();
    // Connect to all previously saved known peers
    for known_peer in known_peers.into_iter().take(MAX_PEERS) {
        let address = known_peer.ip();
        dial_address(address, &mut swarm);
    }

    // TODO: On exit or new peer connection save to file with ip from

    // TODO: Cleanup cloning and things here if possible
    let mut peer_ids: Vec<PeerId> = swarm.all_peers().map(|peer_id| peer_id.clone()).collect();
    let peers_data = peer_ids.iter().map(|peer_id| PeerData::new(peer_id, &mut swarm)).collect();
    save_known_peers(peers_data);

    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        swarm.publish(&Topic::new(BLOCKCHAIN_TOPIC.into()), "HELLO BLOCKCHAIN".as_bytes());
        swarm.publish(&Topic::new(IDENTIFY_TOPIC.into()), "HELLO IDENTIFY".as_bytes());
        loop {
            println!("swarm has {} peers", swarm.all_peers().count());
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(gossip_event)) => match gossip_event {
                    GossipsubEvent::Message(peer_id, id, message) => {
                        println!("MESSAGE");
                        let topic = message.topics.first().unwrap().as_str();
                        if topic == BLOCKCHAIN_TOPIC {
                            println!("BLOCKCHAIN TRANSACTION RECEIVED {}", str::from_utf8(message.data.as_slice()).expect("Failed to create a string from byte data from message"));
                            // Parse incoming message as a block
                            // let transaction = serde_json::from_slice(message.data.as_slice()).expect("Failed to parse incoming message");
                        } else if topic == IDENTIFY_TOPIC {
                            // Parse incoming message as PeerData
                            // let json = serde_json::from_slice(message.data.as_slice()).expect("Failed to parse incoming message");
                        }

                        process_data_stream(str::from_utf8(&message.data).unwrap(), id.to_string(), peer_id.to_string())
                    },
                    _ => {}
                },
                Poll::Ready(None) | Poll::Pending => break,
            }
        }

        if !listening {
            for address in libp2p::Swarm::listeners(&swarm) {
                println!("Listening on {:?}", address);
                listening = true;
            }
        }

        Poll::Pending
    }))
}