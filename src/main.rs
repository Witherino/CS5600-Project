mod blockchain;
use serde_json::{Result, Value, to_string};
use crate::blockchain::{Blockchain, Transaction};
use serde::ser::{Serialize, SerializeStruct, Serializer};

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
    for block in blockchain.block_chain() {
        println!("Block {:?}", block);
    }
}
