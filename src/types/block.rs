use crate::types::{Hash, Transaction};

#[derive(Default, Debug)]
pub struct Block {
    nonce: u128,
    hash: Hash,
    prev_hash: Option<Hash>,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(prev_hash: Option<Hash>) -> Self {
        Block {
            prev_hash,
            ..Default::default()
        }
    }

    pub fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn hash(&self) -> Hash {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TransactionData;

    #[test]
    fn test_creation() {
        let mut block = Block::new(None);
        let mut tx = Transaction::new(TransactionData::CreateAccount("alice".to_string()), None);
        block.set_nonce(1);
        block.add_transaction(tx);

        dbg!(block);
    }
}
