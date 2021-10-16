use crate::types::{AccountId, Block, Blockchain, Error, Transaction, TransactionData};
use blake2::{Blake2s, Digest};
use rand::Rng;

pub fn generate_account_id() -> AccountId {
    let mut rng = rand::thread_rng();
    let seed: u128 = rng.gen();

    hex::encode(Blake2s::digest(&seed.to_be_bytes()))
}

pub fn append_block(bc: &mut Blockchain, nonce: u128) -> Block {
    let mut block = Block::new(bc.get_last_block_hash());
    let tx_create_account =
        Transaction::new(TransactionData::CreateAccount(generate_account_id()), None);
    block.set_nonce(nonce);
    block.add_transaction(tx_create_account);
    let block_clone = block.clone();

    assert!(bc.append_block(block).is_ok());

    block_clone
}

pub fn append_block_with_tx(
    bc: &mut Blockchain,
    nonce: u128,
    transactions: Vec<Transaction>,
) -> Result<(), Error> {
    let mut block = Block::new(bc.get_last_block_hash());
    block.set_nonce(nonce);

    for tx in transactions {
        block.add_transaction(tx);
    }

    bc.append_block(block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        dbg!(generate_account_id());
    }
}
