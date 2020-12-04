mod blockchain;
use serde_json::Value;
use crate::blockchain::{Blockchain, Transaction};
extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;
use std::cell::RefCell;

#[derive(Default, NwgUi)]
pub struct MessageBank {
    
    #[nwg_control(size:(600, 600), position:(800, 300), title: "P2P Money Sender")]
    #[nwg_events( OnWindowClose: [MessageBank::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, max_row: Some(6), spacing: 3)]
    layout: nwg::GridLayout,

    #[nwg_control(text: "Add Peer ID", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, col_span: 1)]
    #[nwg_events( OnButtonClick: [MessageBank::add_peer])]
    add_message_btn: nwg::Button,

    #[nwg_control(text: "Specify $ Amount", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 1, col_span: 1)]
    add_money_btn: nwg::Button,

    //Add peer ID input
    #[nwg_control]
    #[nwg_layout_item(layout: layout, col: 1, row: 0, col_span: 1)]
    message_title: nwg::TextInput,
    
    //Specify $ amount input
    #[nwg_control]
    #[nwg_layout_item(layout: layout, col: 1, row: 1, col_span: 1)]
    message_content: nwg::TextInput,

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

    #[nwg_control(text:"5000")]
    #[nwg_layout_item(layout: layout, col: 3, row: 1, col_span: 1)]
    curr_balance: nwg::Label,

    buttons: RefCell<Vec<nwg::Label>>,
    handlers: RefCell<Vec<nwg::EventHandler>>,
}

impl MessageBank {

    fn add_peer(&self) {
        let title = self.message_title.text();
        let content = self.message_content.text();

        let mut new_label = Default::default();
        nwg::Label::builder()
            .text(&title)
            .parent(&self.window)
            .build(&mut new_label)
            .expect("Failed to build button");

        let mut buttons = self.buttons.borrow_mut();
        let mut handlers = self.handlers.borrow_mut();

        let blen = buttons.len() as u32;
        let (x, y) = (1+(blen % 2), 2+(blen / 2));
        self.layout.add_child(x, y+1, &new_label);

        buttons.push(new_label);
    }

    fn send_money(&self)
    {
        //convert balance to int
        let curr_bal = self.curr_balance.text();
        let i: i32 = curr_bal.parse().unwrap_or(0);

        let id = self.message_title.text();

        //convert sent $ to int
        let amount = self.message_content.text();
        let j: i32 = amount.parse().unwrap_or(0);

        //update balance
        let result = i - j;
        self.curr_balance.set_text(&(result.to_string()));
        
        println!("This is the ID: {}, and this is the amount: {}", id, amount);
    }

    fn exit(&self) {
        let handlers = self.handlers.borrow();
        for handler in handlers.iter() {
            nwg::unbind_event_handler(&handler);
        }
        
        nwg::stop_thread_dispatch();
    }

}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _ui = MessageBank::build_ui(Default::default()).expect("Failed to build UI");
    
    nwg::dispatch_thread_events();
}

fn main() {
    // Create a new blockchain (we would ideally read from disk and update from P2P)
    let mut blockchain = Blockchain::new(2);
    // Peers (peer id) doing transaction
    let alice = 33;
    let bob = 49;
    // Create the transaction
    let transaction = Transaction::new(alice, bob, 100);
    // Add the transaction to the blockchain
    blockchain.add_transaction(transaction).expect("Failed to add new transaction");
    // Make another transaction in the opposite direction
    blockchain.add_transaction(Transaction::new(bob, alice, 50)).unwrap();
    // Print out all blocks
    let serial = serde_json::to_string(&blockchain).unwrap();
    println!("Testing serialization: {}", serial);
    let deserial: Value = serde_json::from_str(&serial).unwrap();
    println!("Testing deserialization: {}", deserial);
    for block in blockchain.block_chain() {
        println!("Block {:?}", block);