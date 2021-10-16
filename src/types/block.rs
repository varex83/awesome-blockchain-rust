use crate::types::{Hash, Transaction};

pub struct Block {
    nonce: u128,
    hash: Hash,
    prev_hash: Hash,
    transactions: Vec<Transaction>,
}
