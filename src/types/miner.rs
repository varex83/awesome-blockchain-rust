use std::time::{SystemTime, UNIX_EPOCH};
use num::BigInt;
use crate::traits::Hashable;
use crate::types::Block;

pub fn mine(block: &mut Block, target: num::BigInt) {
    for nonce in 0..u128::MAX {
        block.set_nonce(nonce);
        if BigInt::parse_bytes(block.hash().as_bytes(), 16).unwrap() < target {
            break;
        }
    }
    block.update_hash();
    block.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u128;
}