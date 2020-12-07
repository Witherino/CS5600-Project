
// Local imports
use crate::blockchain::{Block, GENESIS_BLOCK, Transaction};
// External imports
use serde::{Serialize, Deserialize};
use crate::blockchain::transaction::CurrencyType;
use std::collections::HashMap;
use libp2p::PeerId;

pub const STARTING_BALANCE: CurrencyType = 5000;

#[derive(Debug, Deserialize, Serialize)]
pub struct Blockchain {
    balances: HashMap<String, CurrencyType>,
    block_chain: Vec<Block>,
    difficulty: u8,
}

impl Blockchain {
    fn block_meets_difficulty(&self, block: &Block) -> bool {
        let hash = *block.hash();
        for i in 0..self.difficulty {
            if hash[i as usize] != 0 {
                return false;
            }
        }

        true
    }
    pub fn is_valid_next_block(&self, block: &Block) -> bool {
        let current_block = self.latest_block();

        // TODO: Check that the transaction is valid, sender exists and has funds and receiver exists

        block.previous_hash() == current_block.hash()
        // && block.timestamp() > current_block.timestamp()
        && block.index() > current_block.index()
    }

    pub fn latest_block(&self) -> &Block {
        self.block_chain.last().unwrap()
    }

    pub fn block_chain(&self) -> &[Block] {
        self.block_chain.as_slice()
    }

    pub fn add_peer(&mut self, peer: String) {
        // If we don't have a peer's balance
        if !self.balances.contains_key(&peer) {
            // println!("ADDING NEW PEER {}", peer);
            // Insert the starting balance for that peer
            self.balances.insert(peer, STARTING_BALANCE);
        }
    }

    // Called to add created blocks (from peers or initialized from add_transaction)
    pub fn add_block(&mut self, mut block: Block) {
        while !self.block_meets_difficulty(&block) {
            block.increment_nonce();
        }
        block.update_hash();
        // Make sure we don't skip an index
        assert_eq!(block.index() as usize, self.block_chain.len());
        self.block_chain.push(block.clone());

        // Update balances
        let Transaction { sender, receiver, amount } = block.transaction();
        // HACK: We always add peer here just so they have an entry in the balances table
        self.add_peer(sender.clone());
        // println!("adding block");
        // Update sender balance
        if self.balances.contains_key(&sender) {
            *self.balances.get_mut(&sender).unwrap() -= amount;
        } else {
            panic!("Sending money from a user that doesn't exist");
        }
        // HACK: We always add peer here just so they have an entry in the balances table
        self.add_peer(receiver.clone());
        // Update receiver balance
        if self.balances.contains_key(&receiver) {
            *self.balances.get_mut(&receiver).unwrap() += amount;
        } else {
            panic!("Sending money to a user that doesn't exist");
        }
    }

    pub fn new(difficulty: u8) -> Self {
        Self {
            balances: HashMap::new(),
            block_chain: vec![GENESIS_BLOCK.clone()],
            difficulty
        }
    }

    // Attempt to add a transaction to the blockchain, if successful returns the block (called locally for self created transactions)
    pub fn add_transaction(&mut self, transaction: Transaction) -> Option<&Block> {
        let current_block = self.latest_block();
        let block = Block::new(transaction, current_block.index() + 1, current_block.hash());
        if self.is_valid_next_block(&block) {
            self.add_block(block);

            self.block_chain.last()
        } else {
            None
        }
    }

    pub fn get_balance(&self, peer_id: String) -> CurrencyType {
        // println!("balances {:#?}", self.balances);
        self.balances.get(&peer_id).map(|&bal| bal).unwrap_or(STARTING_BALANCE)
    }
}


