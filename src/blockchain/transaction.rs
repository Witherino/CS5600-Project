
// TODO: Replace PeerId type with LibP2P peer id type
pub type PeerId = u64;

pub type CurrencyType = u64;

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
pub struct Transaction {
    sender: PeerId,
    receiver: PeerId,
    amount: CurrencyType
}

impl Transaction {
    pub fn new(sender: PeerId, receiver: PeerId, amount: CurrencyType) -> Self {
        Self {
            sender,
            receiver,
            amount
        }
    }
    pub fn sender(&self) -> PeerId {
        self.sender
    }
    pub fn receiver(&self) -> PeerId {
        self.receiver
    }
    pub fn amount(&self) -> CurrencyType {
        self.amount
    }
}

pub(in super) const NULL_TRANSACTION: Transaction = Transaction {
    sender: 0,
    receiver: 0,
    amount: 0
};
