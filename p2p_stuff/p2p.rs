use async_std::{io, task};
use futures::{future, prelude::*};
use libp2p::{
    PeerId,
    Swarm,
    NetworkBehaviour,
    identity,
    floodsub::{self, Floodsub, FloodsubEvent},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess
};
use std::{error::Error, task::{Context, Poll}};
use std::env;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/*
pub struct Blockchain {
    block_chain: Vec<Block>,
    difficulty: u8,
}
*/

#[derive(Serialize, Deserialize, Debug)]
struct BlockchainDummy {
    block_chain: i32,
    difficulty: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyPair {
    name: String,
    id: String,
}

fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<KeyPair, Box<Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let u = serde_json::from_reader(reader)?;

    Ok(u)
}


/*
To-do list:
Finish seralize/deserialze methods for KeyPair json and BlockChain json
Have loop poll the block chain json instead of stdin input
Adjust behaviour implementations for the poll change
Implement "Friends List" JSON so user doesnt have to type a specific IP every use
Check other possible behaviours to add such as Identify and Request/Response
*/


fn main() -> Result<(), Box<dyn Error>> {

	// Stuff to store on local host:
	// Discovered peers, host's key pair, personal ledger

    let point = BlockchainDummy { block_chain: 1, difficulty: 2};

    let serialized = serde_json::to_string(&point).unwrap();

    println!("serialized = {}", serialized);

    
    // Checking for exisitng key pair
	let p = env::current_dir().unwrap();
    
    let temp = p.to_string_lossy();
    let mut path = temp.to_string();
    let bar = "/src/key_pair.json".to_string();
    path.push_str(&bar);
    
    let path_present = std::path::Path::new(&path).exists();

    let u = read_user_from_file(path).unwrap();
	// On first run, create a random identity keypair for the local node

	//if !path_present{
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        //let my_name = "MyName";
        //let myPair = { name: my_name, id: local_peer_id};
	//}	
	//else {
        //let temp = read_user_from_file(path).unwrap();
        //println!()
    //}

    println!("Local peer id: {:?}", local_peer_id);
    

	let transport = libp2p::build_development_transport(local_key.clone())?;

    let floodsub_topic = floodsub::Topic::new("block_chain");

    #[derive(NetworkBehaviour)]
    struct BlockBehaviour {
        floodsub: Floodsub,
        mdns: Mdns,

        #[behaviour(ignore)]
        #[allow(dead_code)]
        ignored_member: bool,
    }

    impl NetworkBehaviourEventProcess<FloodsubEvent> for BlockBehaviour {
        // Called when `floodsub` produces an event.
        fn inject_event(&mut self, message: FloodsubEvent) {
            if let FloodsubEvent::Message(message) = message {
                println!("Received: '{:?}' from {:?}", String::from_utf8_lossy(&message.data), message.source);
            }
        }
    }

    impl NetworkBehaviourEventProcess<MdnsEvent> for BlockBehaviour {
        // Called when `mdns` produces an event.
        fn inject_event(&mut self, event: MdnsEvent) {
            match event {
                MdnsEvent::Discovered(list) =>
                    for (peer, _) in list {
                        self.floodsub.add_node_to_partial_view(peer);
                    }
                MdnsEvent::Expired(list) =>
                    for (peer, _) in list {
                        if !self.mdns.has_node(&peer) {
                            self.floodsub.remove_node_from_partial_view(&peer);
                        }
                    }
            }
        }
    }

    let mut swarm = {
        let mdns = Mdns::new()?;
        let mut behaviour = BlockBehaviour {
            floodsub: Floodsub::new(local_peer_id.clone()),
            mdns,
            ignored_member: false,
        };

        behaviour.floodsub.subscribe(floodsub_topic.clone());
        Swarm::new(transport, behaviour, local_peer_id)
    };

    let mut stdin = io::BufReader::new(io::stdin()).lines();



	if let Some(addr) = std::env::args().nth(1) {
        let remote = addr.parse()?;
        Swarm::dial_addr(&mut swarm, remote)?;
        println!("Dialed {}", addr)
    }

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;


	let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => swarm.floodsub.publish(floodsub_topic.clone(), line.as_bytes()),
                Poll::Ready(None) => panic!("Stdin closed"),
                Poll::Pending => break
            }
        }
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => println!("{:?}", event),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            println!("Listening on {:?}", addr);
                            listening = true;
                        }
                    }
                    break
                }
            }
        }
        Poll::Pending
    }))
}