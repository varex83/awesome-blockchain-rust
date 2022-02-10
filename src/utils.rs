use crate::types::{
    miner, AccountId, Balance, Block, Blockchain, Error, Transaction, TransactionData,
};
use blake2::{Blake2s, Digest};
use num::BigInt;

pub fn generate_account_id() -> (AccountId, ed25519_dalek::Keypair) {
    let keypair = ed25519_dalek::Keypair::generate(&mut rand::rngs::OsRng {});
    let public = keypair.public;
    let account_id = hex::encode(Blake2s::digest(public.as_ref()));
    (account_id, keypair)
}

pub fn append_block(bc: &mut Blockchain) -> Block {
    let mut block = Block::new(bc.get_last_block_hash(), bc.get_last_block_number());

    let (account, keypair) = generate_account_id();

    let tx_create_account = Transaction::new(
        TransactionData::CreateAccount {
            account_id: account,
            public_key: keypair.public,
        },
        None,
    );

    block.add_transaction(tx_create_account);

    miner::mine(&mut block, bc.get_latest_target());

    let block_clone = block.clone();

    assert!(bc.append_block(block).is_ok());

    block_clone
}

pub fn append_block_with_tx(
    bc: &mut Blockchain,
    nonce: u128,
    transactions: Vec<Transaction>,
) -> Result<(), Error> {
    let mut block = Block::new(bc.get_last_block_hash(), bc.get_last_block_number());
    block.set_nonce(nonce);

    for tx in transactions {
        block.add_transaction(tx);
    }

    miner::mine(&mut block, bc.get_latest_target());

    bc.append_block(block)
}

pub fn to_compact_format(target: BigInt) -> String {
    let mut hex_target = format!("{:03x}", target);

    let mut len = hex_target.len();

    if len % 2 != 0 {
        hex_target = "0".to_string() + &*hex_target;
        len += 1;
    }

    hex_target = hex_target[0..6].parse().unwrap();
    let target_compacted = format!("0x{:x}{}", len / 2, hex_target);
    return target_compacted;
}

pub fn create_accounts_and_transfer(
    bc: &mut Blockchain,
    account_1: AccountId,
    account_2: AccountId,
    from: AccountId,
    to: AccountId,
    amount_to_mint: Balance,
    amount_to_send: Balance,
    account_1_keypair: &ed25519_dalek::Keypair,
    account_2_keypair: &ed25519_dalek::Keypair,
) -> Result<(), Error> {
    let mut transfer_tx = Transaction::new(
        TransactionData::Transfer {
            to,
            amount: amount_to_send,
        },
        Some(from),
    );

    transfer_tx.sign(account_1_keypair);

    append_block_with_tx(
        bc,
        1,
        vec![
            Transaction::new(
                TransactionData::CreateAccount {
                    account_id: account_1.clone(),
                    public_key: account_1_keypair.public,
                },
                None,
            ),
            Transaction::new(
                TransactionData::CreateAccount {
                    account_id: account_2.clone(),
                    public_key: account_2_keypair.public,
                },
                None,
            ),
            Transaction::new(
                TransactionData::MintInitialSupply {
                    to: account_1.clone(),
                    amount: amount_to_mint,
                },
                None,
            ),
            transfer_tx,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_generate() {
        dbg!(generate_account_id());
    }

    #[test]
    fn test_target_compacted() {
        dbg!(to_compact_format(
            BigInt::from_str("161578008857017275969393492955354620126364423170461532160").unwrap()
        ));
    }
}
