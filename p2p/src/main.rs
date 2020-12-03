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
use isahc::prelude::*;
use std::{
    error::Error,
    task::{Context, Poll},
};


#[derive(Serialize, Deserialize, Debug)]
struct Identity {
    name: String,
    ip: String,
}

fn process_data_stream(msg: &str, _id: String, _peer_id: String) {

    //finding the path of the peers json
    let p = env::current_dir().unwrap();    
    let temp = p.to_string_lossy();
    let mut path = temp.to_string();
    let bar = "/peer_ids.json".to_string();
    path.push_str(&bar);

    let path_present = std::path::Path::new(&path).exists();

    //importing new peer information
    if path_present{
        let mut contents = String::new();
        let mut file = OpenOptions::new().append(true).read(true).open(path).expect("File open failed");
        file.read_to_string(&mut contents).expect("File could not be read");
        if !(contents.contains(msg)){
            file.write_all(msg.as_bytes()).expect("write failed");
        }
    }
    else{
        let mut f = File::create(path).expect("Unable to create file");
        f.write_all(msg.as_bytes()).expect("Unable to write data");
    }
}

//Makes sure files containing more than one JSON object are formatted correctly
fn assure_json_compatibility(filepath: String) -> String{


    let mut file = File::open(filepath).expect("File could not be opened");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("File could not be read");

    let mut clean_json = String::from("[");

    //adding beggining/end brackets, removing unneccesary commas

    for i in 0..contents.len(){
        if i == contents.len()-1{
            if contents.chars().nth(i).unwrap() == '}'{
                clean_json.push('}')
            }
            clean_json.push(']');
        }
        else{
            clean_json.push(contents.chars().nth(i).unwrap())
        }     
    }
    return clean_json;
}

//Format an IP with the correct port and protocol information
fn format_ip(ip: String) -> String{
    let mut formatted = String::from("/ip4/");
    formatted.push_str(&ip);
    formatted.push_str("/tcp/4000");
    return formatted;
}


fn main() -> Result<(), Box<dyn Error>> {
    
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let p = env::current_dir().unwrap();
   
    //Checking for json files
    let temp = p.to_string_lossy();
    let mut my_path = temp.to_string();
    let mut peer_path = temp.to_string();
    let my_bar = "/my_identity.json".to_string();
    let peer_bar = "/peer_ids.json".to_string();
    my_path.push_str(&my_bar);
    peer_path.push_str(&peer_bar);

    let my_path_present = std::path::Path::new(&my_path).exists();
    let peer_path_present = std::path::Path::new(&peer_path).exists();

    //My Identity information
    let mut me = Identity {
        name: "N/A".to_string(),
        ip: "N/A".to_string(),
    };

    //Create a my_identity json file if needed, else read from current
    if my_path_present{
        let mut file = File::open(my_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        me = serde_json::from_str(&contents).expect("JSON incorrectly formatted");
        println!{"Local peer ip: {:?}", me.ip}
    }
    else
    {
        let mut input = String::new();
        println!("No identity saved. Enter your name please:");
        std::io::stdin().read_line(&mut input).unwrap();
        input.pop();
        println!("Hello {}", input);

        let mut response = isahc::get("https://icanhazip.com/")?;
        let mut local_ip = response.text()?;
        local_ip.pop();

        me = Identity {
            name: input,
            ip: local_ip,
        };

        let serialized = serde_json::to_string(&me).unwrap();
        fs::write(my_path, serialized).expect("Unable to write file");
    }

    //Encapsulate identity info being sent to the peer
    let mut me_encoded = serde_json::to_string(&me).unwrap();
    me_encoded.push(',');

    //Generating Key pair
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
        let formatted = format_ip(to_dial);
        let dialing = formatted.clone();
        match formatted.parse() {
            Ok(formatted) => match libp2p::Swarm::dial_addr(&mut swarm, formatted) {
                Ok(_) => println!("Dialed {:?}", dialing),
                Err(e) => println!("Dial {:?} failed: {:?}", dialing, e),
            },
            Err(err) => println!("Failed to parse address to dial: {:?}", err),
        }
    }

    //Loop through established peers to connect too
    if peer_path_present{
        let raw = assure_json_compatibility(peer_path);
        let x: Vec<Identity> = ::serde_json::from_str(&raw)?;
        for i in x {
            let formatted = format_ip(i.ip);
            let dialing = formatted.clone();
            match formatted.parse() {
                Ok(formatted) => match libp2p::Swarm::dial_addr(&mut swarm, formatted) {
                    Ok(_) => println!("Dialed {} at {:?}", i.name, dialing),
                    Err(e) => println!("Dial {:?} failed: {:?}", dialing, e),
                },
                Err(err) => println!("Failed to parse address to dial: {:?}", err),
            }
        }
    }
    

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();


    // Kick it off
    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            if let Err(e) = match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => swarm.publish(&topic, me_encoded.as_bytes()),
                Poll::Ready(None) => panic!("Stdin closed"),
                Poll::Pending => break,
            } {
                println!("Publish error: {:?}", e);
            }
        }

        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(gossip_event)) => match gossip_event {
                    GossipsubEvent::Message(peer_id, id, message) => process_data_stream(str::from_utf8(&message.data).unwrap(), 
                    id.to_string(), peer_id.to_string()),
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