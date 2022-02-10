use crate::traits::Hashable;
use crate::types::{Hash, Transaction};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};
use num::BigInt;

#[derive(Default, Debug, Clone)]
pub struct Block {
    pub nonce: u128,
    pub block_number: u128,
    pub timestamp: u128,
    pub(crate) hash: Option<Hash>,
    pub(crate) prev_hash: Option<Hash>,
    pub(crate) transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(prev_hash: Option<Hash>, prev_block_number: Option<u128>) -> Self {
        let mut block = Block {
            prev_hash,
            block_number: match prev_block_number {
                None => 0,
                Some(num) => num + 1,
            },
            ..Default::default()
        };

        block.update_hash();

        block
    }

    pub fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
        self.update_hash();
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
        self.update_hash();
    }

    pub fn verify(&self, target: num::BigInt) -> bool {
        let _hash = BigInt::parse_bytes(self.hash().as_bytes(), 16).unwrap();

        matches!(&self.hash, Some(hash) if true
                && hash == &self.hash()
                && _hash < target
        )
    }

    pub(crate) fn update_hash(&mut self) {
        self.hash = Some(self.hash());
    }
}

impl Hashable for Block {
    fn hash(&self) -> Hash {
        let mut hasher = Blake2s::new();
        hasher.update(
            format!(
                "{:?}",
                (self.prev_hash.clone(), self.nonce, self.block_number)
            )
            .as_bytes(),
        );
        for tx in self.transactions.iter() {
            hasher.update(tx.hash())
        }

        hex::encode(hasher.finalize_fixed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{miner, TransactionData};
    use crate::utils;

    #[test]
    fn test_creation() {
        let mut block = Block::new(None, None);

        let (account_id, keypair) = utils::generate_account_id();

        let tx = Transaction::new(
            TransactionData::CreateAccount {
                account_id,
                public_key: keypair.public,
            },
            None,
        );
        block.set_nonce(1);
        block.add_transaction(tx);

        dbg!(block);
    }

    #[test]
    fn test_hash() {
        let mut block = Block::new(None, None);

        let (account_alice, keypair_alice) = utils::generate_account_id();

        let tx = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_alice,
                public_key: keypair_alice.public,
            },
            None,
        );

        block.set_nonce(1);

        let hash1 = block.hash();

        block.add_transaction(tx);
        block.set_nonce(1);
        let hash2 = block.hash();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_mine() {
        let mut block = Block::new(None, None);

        let (account_alice, keypair_alice) = utils::generate_account_id();

        let _tx = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_alice,
                public_key: keypair_alice.public,
            },
            None,
        );

        let target = BigInt::from(5) * BigInt::from(10).pow(73);

        // dbg!(&target);
        // dbg!(BigInt::parse_bytes(&block.hash().as_bytes(), 16));

        block.add_transaction(_tx);

        miner::mine(&mut block, target.clone());

        assert!(BigInt::parse_bytes(&block.hash().as_bytes(), 16).unwrap() < target);

        // dbg!(block.nonce);
    }
}
