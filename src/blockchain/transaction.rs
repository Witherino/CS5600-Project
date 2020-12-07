
use serde::{Serialize, Deserialize};
use libp2p::PeerId;

type PeerIdString = String;

pub type CurrencyType = u64;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: PeerIdString,
    pub receiver: PeerIdString,
    pub amount: CurrencyType
}

impl Transaction {
    pub fn new(sender: PeerId, receiver: PeerId, amount: CurrencyType) -> Self {
        Self {
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            amount
        }
    }
    pub fn sender(&self) -> PeerIdString {
        self.sender.clone()
    }
    pub fn receiver(&self) -> PeerIdString {
        self.receiver.clone()
    }
    pub fn amount(&self) -> CurrencyType {
        self.amount
    }
}

pub(in super) const NULL_TRANSACTION: Transaction = Transaction {
    sender: String::new(),
    receiver: String::new(),
    amount: 0
};
