use async_std::{io, task};
use env_logger::{Builder, Env};
use futures::prelude::*;
use libp2p::gossipsub::protocol::MessageId;
use libp2p::gossipsub::{GossipsubEvent, GossipsubMessage, MessageAuthenticity, Topic};
use libp2p::{gossipsub, identity, PeerId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::fs;
use std::env;
use std::str;
//use pnet::datalink;
use isahc::prelude::*;
use std::{
    error::Error,
    task::{Context, Poll},
};


#[derive(Serialize, Deserialize, Debug)]
struct MyIdentity {
    name: String,
    ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockChainDummy {
    block_chain: i32,
    difficulty: i32,
}

fn process_data_stream(msg: &str, _id: String, _peer_id: String) {
    println!("This is the message: {}", msg);

    let p = env::current_dir().unwrap();    
    let temp = p.to_string_lossy();
    let mut path = temp.to_string();
    let bar = "/src/peer_ids.json".to_string();
    path.push_str(&bar);

    //fs::write(path, msg).expect("Unable to write file");
    let mut file = OpenOptions::new().append(true).open(path).expect("file open failed");
    file.write_all(msg.as_bytes()).expect("write failed");
}

fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let p = env::current_dir().unwrap();
    
    let temp = p.to_string_lossy();
    let mut path = temp.to_string();
    let bar = "/src/my_identity.json".to_string();
    path.push_str(&bar);

    let path_present = std::path::Path::new(&path).exists();

    let jtemp = BlockChainDummy {
        block_chain: 1,
        difficulty: 2,
    };

    let j = serde_json::to_string(&jtemp).unwrap();

    if path_present{
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let stuff: MyIdentity = serde_json::from_str(&contents).expect("JSON incorrectly formatted");
        println!{"Local peer ip: {:?}", stuff.ip}
        //j = serde_json::to_string(&stuff)?;
    }
    else
    {
        let mut input = String::new();
        println!("No identity saved. Enter your name please:");
        std::io::stdin().read_line(&mut input).unwrap();
        input.pop();
        println!("Hello {}", input);

        //let mut local_ip: String = "Not detected".to_string();

        let mut response = isahc::get("https://icanhazip.com/")?;
        let mut local_ip = response.text()?;
        local_ip.pop();

        //throws an error if theres a link established on the device 
        //that is not currently connected 
        /*
        for iface in datalink::interfaces() {
            let mut raw = iface.ips[0].to_string();
            let split: Vec<&str> = raw.split("/").take(1).collect::<Vec<_>>();
            let s: String = split.into_iter().collect();
            if s != "127.0.0.1" {
                local_ip = s;
                break;
            }
        }
        */

        let me = MyIdentity {
            name: input,
            ip: local_ip,
        };

        let serialized = serde_json::to_string(&me).unwrap();
        fs::write(path, serialized).expect("Unable to write file");
    }

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::build_development_transport(local_key.clone())?;

    // Create a Gossipsub topic
    let topic = Topic::new("Blockchain-P2P".into());

    // Create a Swarm to manage peers and events
    let mut swarm = {
        // to set default parameters for gossipsub use:
        // let gossipsub_config = gossipsub::GossipsubConfig::default();

        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        // set custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::new()
            .heartbeat_interval(Duration::from_secs(10))
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
            //same content will be propagated.
            .build();
        // build a gossipsub network behaviour
        let mut gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config);
        gossipsub.subscribe(topic.clone());
        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    // Listen on all interfaces and whatever port the OS assigns
    libp2p::Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/4000".parse().unwrap()).unwrap();

    // Reach out to another node if specified
    if let Some(to_dial) = std::env::args().nth(1) {
        let dialing = to_dial.clone();
        match to_dial.parse() {
            Ok(to_dial) => match libp2p::Swarm::dial_addr(&mut swarm, to_dial) {
                Ok(_) => println!("Dialed {:?}", dialing),
                Err(e) => println!("Dial {:?} failed: {:?}", dialing, e),
            },
            Err(err) => println!("Failed to parse address to dial: {:?}", err),
        }
    }

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();


    // Kick it off
    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            if let Err(e) = match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => swarm.publish(&topic, j.as_bytes()),
                Poll::Ready(None) => panic!("Stdin closed"),
                Poll::Pending => break,
            } {
                println!("Publish error: {:?}", e);
            }
        }

        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(gossip_event)) => match gossip_event {
                    GossipsubEvent::Message(peer_id, id, message) => process_data_stream(str::from_utf8(&message.data).unwrap(), id.to_string(), peer_id.to_string()),
                    /*println!(
                        "Got message: {} with id: {} from peer: {:?}",
                        String::from_utf8_lossy(&message.data),
                        id,
                        peer_id
                    )*/
                    _ => {}
                },
                Poll::Ready(None) | Poll::Pending => break,
            }
        }

        if !listening {
            for addr in libp2p::Swarm::listeners(&swarm) {
                println!("Listening on {:?}", addr);
                listening = true;
            }
        }

        Poll::Pending
    }))
}