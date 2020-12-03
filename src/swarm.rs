
// Std imports
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
// External imports
use libp2p::gossipsub::{GossipsubMessage, GossipsubConfigBuilder, MessageAuthenticity, Gossipsub, MessageId, Topic};
use libp2p::{Swarm, PeerId};
use libp2p::identity::Keypair;
use libp2p::core::Multiaddr;

pub const BLOCKCHAIN_TOPIC: &'static str = "blockchain";
pub const IDENTIFY_TOPIC: &'static str = "identify";

// What aspect of a message makes it unique (that way we don't repeat unnecessarily)
fn message_hasher(message: &GossipsubMessage) -> MessageId {
    // TODO: Just use block hash instead of message hash


    // Hash the message to get an unique id for it
    let mut s = DefaultHasher::new();
    message.data.hash(&mut s);
    MessageId::from(s.finish().to_string())
}

pub fn spawn_swarm(keypair: Keypair, peer_id: PeerId) -> Swarm<Gossipsub> {
    // How we verify who sent a message
    let auth = MessageAuthenticity::Signed(keypair.clone());

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::build_development_transport(keypair).expect("Failed to create transport channel");

    // Create a configuration for the network on how to handle messages, timeouts, peers, and more
    let config = GossipsubConfigBuilder::new()
        .message_id_fn(message_hasher)
        .build();

    // Create the network behavior given the auth method and config
    let mut behavior = Gossipsub::new(auth, config);

    // The blockchain topic, where all new transactions are transported
    let blockchain_topic = Topic::new(BLOCKCHAIN_TOPIC.into());
    behavior.subscribe(blockchain_topic);

    // Identify topic when a new peer connects
    let identify_topic = Topic::new(IDENTIFY_TOPIC.into());
    behavior.subscribe(identify_topic);

    libp2p::Swarm::new(transport, behavior, peer_id)
}

pub fn dial_address(address: Multiaddr, swarm: &mut Swarm<Gossipsub>) {
    match libp2p::Swarm::dial_addr(swarm, address.clone()) {
        Ok(_) => println!("Dialed {:?}", address.to_string()),
        Err(e) => eprintln!("Dial {:?} failed: {:?}", address.to_string(), e),
    }
}
