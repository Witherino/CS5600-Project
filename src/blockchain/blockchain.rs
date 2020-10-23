use crate::blockchain::{Block, GENESIS_BLOCK, Transaction};
use serde::{Serialize, Deserialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct Blockchain {
    block_chain: Vec<Block>,
    difficulty: u8,
}

// impl Serialize for Blockchain {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//     {
//         let mut blockchain = serializer.serialize_struct("Blockchain", 2)?;
//         blockchain.serialize_field("block_chain", &self.block_chain)?;
//         blockchain.serialize_field("difficulty", &self.difficulty)?;
//         blockchain.end()
//     }
// }

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
        && block.timestamp() > current_block.timestamp()
        && block.index() > current_block.index()
    }

    pub fn latest_block(&self) -> &Block {
        self.block_chain.last().unwrap()
    }

    pub fn block_chain(&self) -> &[Block] {
        self.block_chain.as_slice()
    }

    fn add_block(&mut self, mut block: Block) {
        while !self.block_meets_difficulty(&block) {
            block.increment_nonce();
        }
        block.update_hash();
        // Make sure we don't skip an index
        assert_eq!(block.index() as usize, self.block_chain.len());
        self.block_chain.push(block);
    }

    pub fn new(difficulty: u8) -> Self {
        Self {
            block_chain: vec![GENESIS_BLOCK.clone()],
            difficulty
        }
    }

    // Attempt to add a transaction to the blockchain, if successful returns the block
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
}

