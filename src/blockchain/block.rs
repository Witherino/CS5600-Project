
use lazy_static::*;
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use crate::blockchain::transaction::{Transaction, NULL_TRANSACTION};

const HASH_SIZE: usize = 32;
const NULL_HASH: &'static [u8; HASH_SIZE] = b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[derive(Debug, Clone)]
pub struct Block {
    index: u64,
    timestamp: DateTime<Utc>,
    hash: Arc<[u8; HASH_SIZE]>,
    previous_hash: Arc<[u8; HASH_SIZE]>,
    transaction: Transaction,
    nonce: u64,
}

impl Block {
    // Update hash member from other members
    pub fn update_hash(&mut self) {
        let mut hasher = Sha256::new();
        // Hash the timestamp (only supports 584 years and will break around the year 2600)
        hasher.update(self.timestamp().timestamp_nanos().to_le_bytes());
        // Hash the transaction
        hasher.update(self.transaction().sender().to_le_bytes());
        hasher.update(self.transaction().receiver().to_le_bytes());
        hasher.update(self.transaction().amount().to_le_bytes());
        // Include previous hash in hash
        hasher.update(self.previous_hash().as_ref());
        // Include index
        hasher.update(self.index().to_le_bytes());
        // Include nonce
        hasher.update(self.nonce().to_le_bytes());

        let hash = hasher.finalize();
        let mut byte_array = [0u8; HASH_SIZE];
        for i in 0..HASH_SIZE {
            let t = hash.as_slice()[i];
            byte_array[i] = t;
        }
        self.hash = Arc::new(byte_array);
    }
    // Constructor
    pub fn new(transaction: Transaction, index: u64, previous_hash: Arc<[u8; HASH_SIZE]>) -> Self {
        let timestamp = Utc::now();
        let mut block = Self {
            hash: Arc::new(*NULL_HASH),
            nonce: 0,
            index,
            previous_hash,
            timestamp,
            transaction,
        };
        // Set the block's hash from its properties
        block.update_hash();

        block
    }
    // Setters
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
        self.update_hash();
    }
    // Getters
    pub fn hash(&self) -> Arc<[u8; HASH_SIZE]> {
        self.hash.clone()
    }
    pub fn previous_hash(&self) -> Arc<[u8; HASH_SIZE]> {
        self.previous_hash.clone()
    }
    pub fn transaction(&self) -> Transaction {
        self.transaction
    }
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
    pub fn index(&self) -> u64 {
        self.index
    }
}

lazy_static! {
    pub static ref GENESIS_BLOCK: Block = {
        let mut block = Block {
            hash: Arc::new(*NULL_HASH),
            nonce: 0, // NOTE: We might want to find a nonce that actually makes the first few digits zero (but it isn't important since this is just the root)
            index: 0,
            previous_hash: Arc::new(*NULL_HASH),
            transaction: NULL_TRANSACTION,
            timestamp: DateTime::parse_from_rfc3339("2020-10-11T08:49:15Z").unwrap().with_timezone(&Utc),
        };
        // Set the block's hash from its properties
        block.update_hash();

        block
    };
}
