// Modules
mod swarm;
mod peer_data;
mod blockchain;

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
// Local imports
use crate::swarm::{spawn_swarm, BLOCKCHAIN_TOPIC, IDENTIFY_TOPIC, dial_address};
use crate::peer_data::{get_keypair, get_known_peers, save_known_peer, PeerData, save_known_peers};
use crate::blockchain::*;
// Std imports
use std::str;
use std::env;
use std::{
    error::Error,
    task::{Context, Poll},
};
// External imports
use futures::prelude::*;
use async_std::{task, io};
use libp2p::gossipsub::{GossipsubEvent, Topic, Gossipsub};
use serde::{Serialize, Deserialize};
use futures::StreamExt;
use libp2p::Swarm;
use libp2p::swarm::NetworkBehaviour;
use libp2p::core::Multiaddr;
use libp2p::PeerId;
use lazy_static::*;

use std::thread;
use std::sync::mpsc::{Receiver, channel, Sender};
use std::sync::{RwLock, Mutex};

// Gui imports
use nwd::NwgUi;
use nwg::NativeUi;
use std::cell::RefCell;
use libp2p::core::network::Peer;
use std::str::FromStr;

const DEFAULT_PORT: u16 = 4000;
const MAX_PEERS: usize = 10;

// JEFF ADDED
lazy_static! {
    pub static ref BLOCKCHAIN: RwLock<Blockchain> = RwLock::new(Blockchain::new(1));

    pub static ref MY_PEER_ID: PeerId = PeerId::from_public_key(get_keypair().public());

    pub static ref SWARM: Mutex<Swarm<Gossipsub>> = Mutex::new(spawn_swarm(get_keypair(), MY_PEER_ID.clone()));
}

#[derive(Default, NwgUi)]
pub struct MessageBank {
    
    #[nwg_control(size:(650, 600), position:(800, 300), title: "P2P Money Sender")]
    #[nwg_events( OnWindowClose: [MessageBank::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, max_row: Some(6), spacing: 3)]
    layout: nwg::GridLayout,

    #[nwg_control(text: "Add Peer ID", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, col_span: 1)]
    #[nwg_events( OnButtonClick: [MessageBank::add_peer])]
    add_peer_btn: nwg::Button,

    #[nwg_control(text: "Specify $ Amount:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 1, col_span: 1)]
    add_money_btn: nwg::Label,

    //Box to add peer ID
    #[nwg_control]
    #[nwg_layout_item(layout: layout, col: 1, row: 0, col_span: 1)]
    peer_id: nwg::TextInput,
    
    //Box to specify $ amount
    #[nwg_control]
    #[nwg_layout_item(layout: layout, col: 1, row: 1, col_span: 1)]
    sent_amount: nwg::TextInput,

    #[nwg_control(text:"Peer History:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 3, col_span: 1)]
    friend: nwg::Label,

    #[nwg_control(text: "Send", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 2)]
    #[nwg_events( OnButtonClick: [MessageBank::send_money])]
    send_money: nwg::Button,
    
    #[nwg_control(text:"Current Balance: ")]
    #[nwg_layout_item(layout: layout, col: 2, row: 1, col_span: 1)]
    balance: nwg::Label,

    #[nwg_control(text:"10000")]
    #[nwg_layout_item(layout: layout, col: 3, row: 1, col_span: 1)]
    curr_balance: nwg::Label,

    boxes: RefCell<Vec<nwg::CheckBox>>,
    handlers: RefCell<Vec<nwg::EventHandler>>,
}

impl MessageBank {

    pub fn add_peer(&self) {
        let title = self.peer_id.text();

        self.peer_id.set_text("");

        let mut new_check = Default::default();
        nwg::CheckBox::builder()
            .text(&title)
            .parent(&self.window)
            .build(&mut new_check)
            .expect("Failed to build button");
   
        let mut boxes = self.boxes.borrow_mut();

        // new peer box positions are weird
        let blen = boxes.len() as u32;
        let (x, y) = (1+(blen % 3), 2+(blen / 3));
        self.layout.add_child(x, y+1, &new_check);

        boxes.push(new_check);
    }

    pub fn send_money(&self)
    {

        let mut total_amount = 0;
        let mut all_peers: String = "".to_owned();

        //check_state returns a checkbox, not a bool, so this checkbox is being used as a bool to compare the two
        let mut new_check = Default::default();
        nwg::CheckBox::builder()
            .check_state(nwg::CheckBoxState::Checked)
            .parent(&self.window)
            .build(&mut new_check)
            .expect("Failed to build button");

        let boxes = self.boxes.borrow_mut();
        let mut checks: Vec::<String> = Vec::<String>::new();
        for n in 0..boxes.len()
        {
            if boxes[n].check_state().eq(&new_check.check_state())
            {
                checks.push(boxes[n].text());
            }
        }

        if checks.len() == 0
        {
            nwg::simple_message("Error", "Please select at least 1 peer");
        }
        else if self.sent_amount.text().eq("0") || self.sent_amount.text().eq("")
        {
            nwg::simple_message("Error", "Please add an amount to send");
        }
        else if !self.sent_amount.text().chars().all(char::is_numeric)
        {
            nwg::simple_message("Error", "Sent amount must be a postive number");
        }
        else
        {
            let mut positive: bool = true;
            for n in 0..checks.len()
            {
                //convert balance to int
                let curr_bal = self.curr_balance.text();
                let curr_bal_int: i32 = curr_bal.parse().unwrap_or(0);

                //convert sent $ to int
                let sent_amount = self.sent_amount.text();
                let sent_amount_int: i32 = sent_amount.parse().unwrap_or(0);

                //update balance
                let result = curr_bal_int - sent_amount_int;
                if result < 0
                {
                    nwg::simple_message("Error", "Could not complete transaction. Insufficient funds");
                    positive = false;
                    break;
                }
                total_amount += sent_amount_int;

                if checks.len() == 1
                {
                    all_peers.push_str(&checks[n]);
                }
                else if n < checks.len()-1
                {
                    all_peers.push_str(&checks[n]);
                    all_peers.push_str(", ");
                }
                else
                {
                    all_peers.push_str("and ");
                    all_peers.push_str(&checks[n]);
                }

                // JEFF ADDED
                let transaction = Transaction::new(MY_PEER_ID.clone(), PeerId::from_str(&checks[n]).expect("Invalid PeerId"), sent_amount_int as u64);
                // Add transaction to local blockchain
                BLOCKCHAIN.write().unwrap().add_transaction(transaction);
                let serialized_block = serde_json::to_string(BLOCKCHAIN.read().unwrap().latest_block()).expect("Failed to serialize block");
                // Send to peers
                SWARM.lock().unwrap().publish(&Topic::new(BLOCKCHAIN_TOPIC.into()), serialized_block.as_bytes());
                // Set text for balance from blockchain
                self.curr_balance.set_text(BLOCKCHAIN.read().unwrap().get_balance(MY_PEER_ID.to_string()).to_string().as_str());
            }
            if positive
            {
                let mut test_s: String = "Sent total of $".to_owned();
                test_s.push_str(&total_amount.to_string());
                test_s.push_str(" to ");
                test_s.push_str(&all_peers);

                nwg::simple_message("Transaction Successful", &test_s);
            }
        }
        self.peer_id.set_text("");
        self.sent_amount.set_text("");
    }

    fn exit(&self) {
        let handlers = self.handlers.borrow();
        for handler in handlers.iter() {
            nwg::unbind_event_handler(&handler);
        }
        
        nwg::stop_thread_dispatch();
    }

}


fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();

    thread::spawn(move || {

        nwg::init().expect("Failed to init Native Windows GUI");
        nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

        let _ui = MessageBank::build_ui(Default::default()).expect("Failed to build UI");
    
        loop {
            let receive_result = rx.try_recv();
            
            let new_peer: Option<PeerId> = if receive_result.is_ok() {
            Some(rx.recv().unwrap())
            } else {
            None
            };
            //println!("here it is {:?}", new_peer);
            if let Some(ref peer_id) = new_peer {
                _ui.peer_id.set_text(&new_peer.unwrap().to_string()[6..]);
                _ui.add_peer();
            }
            
            nwg::dispatch_thread_events_with_callback(move || {
        })
        }
    });

    println!("Blockchain CS5600");

    println!("Logged in as {}", MY_PEER_ID.to_string());
    
    // Create a Swarm to manage peers and events
    // let mut swarm = spawn_swarm(get_keypair(), MY_PEER_ID.clone());

    // Listen on all interfaces and whatever port the OS assigns
    libp2p::Swarm::listen_on(&mut *SWARM.lock().unwrap(), "/ip4/0.0.0.0/tcp/4000".parse().expect("Invalid swarm ip")).expect("Failed to start swarm listen");

    // Reach out to another node if specified
    if let Some(to_dial) = std::env::args().nth(1) {
        match to_dial.parse() {
            Ok(address) => dial_address(address, &mut *SWARM.lock().unwrap()),
            Err(err) => eprintln!("Failed to parse address to dial: {:?}", err),
        }
    }

    // Get all previously known peers
    let known_peers = get_known_peers();
    // Connect to all previously saved known peers
    for known_peer in known_peers.into_iter().take(MAX_PEERS) {
        let address = known_peer.ip();
        dial_address(address, &mut *SWARM.lock().unwrap());
    }

    // TODO: On exit or new peer connection save to file with ip from

    // TODO: Cleanup cloning and things here if possible
    let mut peer_ids: Vec<PeerId> = swarm.all_peers().map(|peer_id| peer_id.clone()).collect();
    let peers_data = peer_ids.iter().map(|peer_id| PeerData::new(peer_id, &mut swarm)).collect();
    save_known_peers(peers_data);

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => {
                    if line.starts_with("send ") {
                        let mut tokens = line.split_ascii_whitespace();
                        let amount_str = tokens.next().expect("send requires an amount");
                        let receiver_str = tokens.next().expect("send requires a receiver");
                        let amount = amount_str.parse().expect("invalid amount in send (expected a valid number)");
                        let receiver_peer = PeerId::from_str(receiver_str).expect("invalid peer id in send");

                        // Create transaction
                        let transaction = Transaction::new(MY_PEER_ID.clone(), receiver_peer, amount);
                        // Update local blockchain
                        BLOCKCHAIN.write().unwrap().add_transaction(transaction);

                        // Send to the rest of the swarm
                        let serialized_block = serde_json::to_string(BLOCKCHAIN.read().unwrap().latest_block()).expect("Failed to serialize block");
                        SWARM.lock().unwrap().publish(&Topic::new(BLOCKCHAIN_TOPIC.into()), serialized_block.as_bytes());
                    } else if line.starts_with("bal ") {
                        println!("Balance: ${}", BLOCKCHAIN.read().unwrap().get_balance(line[4..].into()));
                    } else if line == "bal" {
                        println!("Balance: ${}", BLOCKCHAIN.read().unwrap().get_balance(MY_PEER_ID.to_string()));
                    } else {
                        eprintln!("Unknown command");
                    }
                },
                Poll::Ready(None) => panic!("Stdin closed"),
                Poll::Pending => break,
            }
        }

        loop {
            match SWARM.lock().unwrap().poll_next_unpin(cx) {
                Poll::Ready(Some(gossip_event)) => match gossip_event {
                    GossipsubEvent::Message(peer_id, id, message) => {
                        let topic = message.topics.first().unwrap().as_str();
                        if topic == BLOCKCHAIN_TOPIC {
                            // Parse incoming message as a block
                            let block = serde_json::from_slice(message.data.as_slice()).expect("Failed to parse incoming message");
                            // Add block
                            BLOCKCHAIN.write().unwrap().add_block(block);
                        }
                        // TODO: Use another topic to send whole blockchain to new peers so they're up to date (if we have time)
                    },
                    GossipsubEvent::Subscribed{peer_id, ..} => {
                        //println!("HERE HUH??? {}", peer_id.to_string());
                        tx.send(peer_id).unwrap();
                    },
                    // GossipsubEvent::Unsubscribed{peer_id, ..} => {
                    //     // REMOVE PEER
                    // },
                    _ => {}
                    
                },
                Poll::Ready(None) | Poll::Pending => break,
            }
        }

        if !listening {
            for address in libp2p::Swarm::listeners(&*SWARM.lock().unwrap()) {
                println!("Listening on {:?}", address);
                listening = true;
            }
        }

        Poll::Pending
    }))

}