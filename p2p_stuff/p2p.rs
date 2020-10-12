use async_std::task;
use futures::{future, prelude::*};
use libp2p::{identity, PeerId, ping::{Ping, PingConfig}, Swarm};
use std::{error::Error, task::{Context, Poll}};
use std::env;


fn main() -> Result<(), Box<dyn Error>> {

	// Stuff to store on local host:
	// Discovered peers, host's key pair, personal ledger
	// Checking for exisitng key pair
	let p = env::current_dir().unwrap();
    
    let temp = p.to_string_lossy();
    let mut path = temp.to_string();
    let bar = "/src/key_pair.txt".to_string();
    path.push_str(&bar);
    
    let path_present = std::path::Path::new(&path).exists();


	// On first run, create a random identity keypair for the local node

	//if !path_present {
		let local_key = identity::Keypair::generate_ed25519();
		let local_peer_id = PeerId::from(local_key.public());
	//}	
	// else implementation for reading key pair file

	println!("Local peer id: {:?}", local_peer_id);

	// Create a transport instance (TCP) with keypair as argument
	// We can "upgrade" the connection with desired protocols if we want to get into that
	let transport = libp2p::build_development_transport(local_key.clone())?;

	// Create a struct that implements the desired NetworkBehaviour trait
	// DummyBehaviour, Gossipsub, Floodsub, and some others listed at the link below
	// https://docs.rs/libp2p/0.28.1/libp2p/swarm/trait.NetworkBehaviour.html
	let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

	// Initiating a swarm with the transport, behaviour, and local peer id
	let mut swarm = Swarm::new(transport, behaviour, local_peer_id);


	// Connect to another peer in the network by using their IP/port as an arg
	// This will be changing for us (probably a stored list of peers on local client)
	if let Some(addr) = std::env::args().nth(1) {
        let remote = addr.parse()?;
        Swarm::dial_addr(&mut swarm, remote)?;
        println!("Dialed {}", addr)
    }

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;


	// Start listening
	let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => println!("{:?}", event),
                Poll::Ready(None) => return Poll::Ready(()),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            println!("Listening on {}", addr);
                            listening = true;
                        }
                    }
                    return Poll::Pending
                }
            }
        }
    }));

    Ok(())

}