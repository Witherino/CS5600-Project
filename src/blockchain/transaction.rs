use serde::{Serialize, Deserialize};
// TODO: Replace PeerId type with LibP2P peer id type
pub type PeerId = u64;

pub type CurrencyType = u64;

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[repr(packed)]
pub struct Transaction {
    sender: PeerId,
    receiver: PeerId,
    amount: CurrencyType
}

// impl Serialize for Transaction {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//     {
//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("Transaction", 3)?;
//         state.serialize_field("sender", &self.sender)?;
//         state.serialize_field("receiver", &self.receiver)?;
//         state.serialize_field("amount", &self.amount)?;
//         state.end()
//     }
// }

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
