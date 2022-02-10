use blockchain_workshop::traits::Hashable;
use blockchain_workshop::types::{mine, Block, Blockchain, Transaction, TransactionData};
use blockchain_workshop::utils::append_block;
use blockchain_workshop::{types, utils};

fn main() {
    let bc = &mut Blockchain::new();

    let (account_1, account_1_keypair) = utils::generate_account_id();
    let (account_2, account_2_keypair) = utils::generate_account_id();

    let mut transfer_tx = Transaction::new(
        TransactionData::Transfer {
            to: account_2.clone(),
            amount: 100_000,
        },
        Some(account_1.clone()),
    );

    transfer_tx.sign(&account_1_keypair);

    let transactions = vec![
        Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_1.clone(),
                public_key: account_1_keypair.public,
            },
            None,
        ),
        Transaction::new(
            TransactionData::MintInitialSupply {
                to: account_1.clone(),
                amount: 100_000_000,
            },
            None,
        ),
    ];

    let mut block = Block::new(bc.get_last_block_hash(), bc.get_last_block_number());

    for tx in transactions {
        block.add_transaction(tx);
    }

    mine(&mut block, bc.get_latest_target());

    println!("Mined block (genesis) with nonce: {:?}", block.nonce);

    assert!(bc.append_block(block).is_ok());

    let transactions = vec![
        Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_2.clone(),
                public_key: account_2_keypair.public,
            },
            None,
        ),
        transfer_tx,
    ];

    let mut block = Block::new(bc.get_last_block_hash(), bc.get_last_block_number());

    for tx in transactions {
        block.add_transaction(tx);
    }

    mine(&mut block, bc.get_latest_target());

    println!("Mined block with nonce: {}", block.nonce);
    // dbg!(bc.append_block(block.clone()));
    assert!(bc.append_block(block.clone()).is_ok());

    for _ in 0..10 {
        append_block(bc);
        println!("Block mined");
    }

    let mut blocktimes: Vec<&types::Block> = vec![];

    for block in bc.blocks.iter() {
        blocktimes.push(block);
    }

    let mut before: u128 = 0;
    let mut average = 0;

    for block in blocktimes.into_iter().rev() {
        if block.block_number == 0 {
            before = block.timestamp;
            continue;
        }

        println!(
            "Block number: \t{} \n \
            Block mining time: \t{} \n \
            Block target: \t{} \n \
            Block hash: \t{} \n \
            Block timestamp: {}\n",
            block.block_number,
            block.timestamp - before,
            utils::to_compact_format(bc.get_target(block.block_number)),
            block.hash(),
            block.timestamp
        );

        average += block.timestamp - before;
        before = block.timestamp;
    }

    println!(
        "Average block_time: {}",
        average / (bc.blocks.len() as u128 - 1)
    );

    // dbg!(bc);
}
