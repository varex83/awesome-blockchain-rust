use crate::traits::{Hashable, WorldState};
use crate::types::{Account, AccountId, AccountType, Block, Chain, Error, Hash, Transaction};
use num::{BigInt, FromPrimitive};
use std::cmp::{max, min};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

const MAX_TARGET_CHANGE: i32 = 4; // x0.25 or x4
const EXPECTED_TIME: i32 = 1000; // 10 millisec

#[derive(Default, Debug)]
pub struct Blockchain {
    blocks: Chain<Block>,
    accounts: HashMap<AccountId, Account>,
    transaction_pool: Vec<Transaction>,
}

impl WorldState for Blockchain {
    fn create_account(
        &mut self,
        account_id: AccountId,
        account_type: AccountType,
        public_key: ed25519_dalek::PublicKey,
    ) -> Result<(), Error> {
        match self.accounts.entry(account_id.clone()) {
            Entry::Occupied(_) => Err(format!("AccountId already exist")),
            Entry::Vacant(v) => {
                let account = Account::new(account_type, public_key);
                v.insert(account);
                Ok(())
            }
        }
    }

    fn get_account_by_id(&self, account_id: AccountId) -> Option<&Account> {
        self.accounts.get(&account_id)
    }

    fn get_account_by_id_mut(&mut self, account_id: AccountId) -> Option<&mut Account> {
        self.accounts.get_mut(&account_id)
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let nbc = Self {
            ..Default::default()
        };
        nbc
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn append_block(&mut self, mut block: Block) -> Result<(), Error> {
        //DONE Task 3: Implement mining
        block.block_number = match self.blocks.head() {
            None => 0,
            Some(x) => x.block_number + 1,
        };

        block.mine(self.get_latest_target());

        if !block.verify(self.get_latest_target()) {
            return Err("Block has invalid hash".to_string());
        }
        let is_genesis = self.blocks.len() == 0;

        if block.transactions.len() == 0 {
            return Err("Block has 0 transactions.".to_string());
        }

        let account_backup = self.accounts.clone();
        for tx in &block.transactions {
            let res = tx.execute(self, is_genesis);
            if let Err(error) = res {
                self.accounts = account_backup;
                return Err(format!("Error during tx execution: {}", error));
            }
        }

        if !block.verify(self.get_latest_target()) {
            return Err("Block has invalid hash".to_string());
        }
        // DONE Task 3: Append block only if block.hash < target
        // Adjust difficulty of target each block generation (epoch)

        self.blocks.append(block);

        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut block_num = self.blocks.len();
        let mut prev_block_hash: Option<Hash> = None;

        for block in self.blocks.iter() {
            let is_genesis = block_num == 1;

            if !block.verify(self.get_target(block.block_number)) {
                return Err(format!("Block {} has invalid hash", block_num));
            }

            if !is_genesis && block.prev_hash.is_none() {
                return Err(format!("Block {} doesn't have prev_hash", block_num));
            }

            if is_genesis && block.prev_hash.is_some() {
                return Err("Genesis block shouldn't have prev_hash".to_string());
            }

            if block_num != self.blocks.len() {
                if let Some(prev_block_hash) = &prev_block_hash {
                    if prev_block_hash != &block.hash.clone().unwrap() {
                        return Err(format!(
                            "Block {} prev_hash doesn't match Block {} hash",
                            block_num + 1,
                            block_num
                        ));
                    }
                }
            }

            prev_block_hash = block.prev_hash.clone();
            block_num -= 1;
        }

        Ok(())
    }

    pub fn get_latest_target(&self) -> BigInt {
        self.get_target(self.blocks.len() as u128)
    }

    pub fn get_target(&self, block_number: u128) -> BigInt {
        let initial_target: BigInt = BigInt::from(5) * BigInt::from(10).pow(73);

        let mut target: BigInt = initial_target;
        let mut prev_timestamp: u128 = 0;

        let mut blocks: Vec<&Block> = vec![];
        for block in self.blocks.iter() {
            blocks.push(block);
        }

        for block in blocks.into_iter().rev() {
            if block.block_number > 0 {
                let mut new_target = target.clone()
                    * BigInt::from_i64(block.timestamp as i64 - prev_timestamp as i64).unwrap()
                    / BigInt::from(EXPECTED_TIME);

                new_target = min(
                    new_target,
                    target.clone() * BigInt::from_i32(MAX_TARGET_CHANGE).unwrap(),
                );
                new_target = max(
                    new_target,
                    target.clone() / BigInt::from_i32(MAX_TARGET_CHANGE).unwrap(),
                );

                target = new_target;
            }

            prev_timestamp = block.timestamp;

            if block.block_number == block_number {
                break;
            }
        }

        target
    }

    pub fn get_last_block_hash(&self) -> Option<Hash> {
        self.blocks.head().map(|block| block.hash())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TransactionData;
    use crate::utils;
    use crate::utils::{append_block, append_block_with_tx};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_new() {
        let bc = Blockchain::new();
        assert_eq!(bc.get_last_block_hash(), None);
    }

    #[test]
    fn test_create_genesis_block() {
        let bc = &mut Blockchain::new();

        let (account, keypair) = utils::generate_account_id();

        let tx_create_account = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account.clone(),
                public_key: keypair.public,
            },
            None,
        );

        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: account.clone(),
                amount: 100_000_000,
            },
            None,
        );
        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account, tx_mint_initial_supply]).is_ok()
        );

        let satoshi = bc.get_account_by_id(account);

        assert!(satoshi.is_some());
        assert_eq!(satoshi.unwrap().balance, 100_000_000);
    }

    #[test]
    fn test_create_genesis_block_fails() {
        let mut bc = Blockchain::new();

        let (account, keypair) = utils::generate_account_id();

        let tx_create_account = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account.clone(),
                public_key: keypair.public,
            },
            None,
        );

        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: account.clone(),
                amount: 100_000_000,
            },
            None,
        );

        let mut block = Block::new(None);
        block.set_nonce(1);
        block.add_transaction(tx_mint_initial_supply);
        block.add_transaction(tx_create_account);

        assert_eq!(
            bc.append_block(block).err().unwrap(),
            "Error during tx execution: Invalid account.".to_string()
        );
    }

    #[test]
    fn test_state_rollback_works() {
        let mut bc = Blockchain::new();

        let (account_satoshi, secret) = utils::generate_account_id();

        let tx_create_account = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_satoshi.clone(),
                public_key: secret.public,
            },
            None,
        );

        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: account_satoshi.clone(),
                amount: 100_000_000,
            },
            None,
        );

        let mut block = Block::new(None);

        block.set_nonce(1);
        block.add_transaction(tx_create_account);
        block.add_transaction(tx_mint_initial_supply);

        assert!(bc.append_block(block).is_ok());

        let (account_alice, keypair_alice) = utils::generate_account_id();
        let (account_bob, keypair_bob) = utils::generate_account_id();

        let mut block = Block::new(bc.get_last_block_hash());
        let tx_create_alice = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_alice.clone(),
                public_key: keypair_alice.public,
            },
            None,
        );
        let tx_create_bob = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_bob.clone(),
                public_key: keypair_bob.public,
            },
            None,
        );

        block.set_nonce(2);
        block.add_transaction(tx_create_alice);
        block.add_transaction(tx_create_bob.clone());
        block.add_transaction(tx_create_bob);

        assert!(bc.append_block(block).is_err());

        assert!(bc.get_account_by_id(account_satoshi).is_some());
        assert!(bc.get_account_by_id(account_alice).is_none());
        assert!(bc.get_account_by_id(account_bob).is_none());
    }

    #[test]
    fn test_validate() {
        let bc = &mut Blockchain::new();

        let (account_satoshi, keypair_satoshi) = utils::generate_account_id();

        let tx_create_account = Transaction::new(
            TransactionData::CreateAccount {
                account_id: account_satoshi.clone(),
                public_key: keypair_satoshi.public,
            },
            None,
        );
        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: account_satoshi.clone(),
                amount: 100_000_000,
            },
            None,
        );
        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account, tx_mint_initial_supply]).is_ok()
        );

        append_block(bc);
        append_block(bc);

        assert!(bc.validate().is_ok());

        let mut iter = bc.blocks.iter_mut();
        iter.next();
        iter.next();
        let block = iter.next().unwrap();

        block.transactions[1].data = TransactionData::MintInitialSupply {
            to: account_satoshi,
            amount: 100,
        };

        assert!(bc.validate().is_err());
    }

    #[test]
    fn test_transfer() {
        let bc = &mut Blockchain::new();

        let (account_alice, alice_keypair) = utils::generate_account_id();
        let (account_bob, bob_keypair) = utils::generate_account_id();

        assert!(utils::create_accounts_and_transfer(
            bc,
            account_alice.clone(),
            account_bob.clone(),
            account_alice.clone(),
            account_bob.clone(),
            100_000_000,
            100_000,
            &alice_keypair,
            &bob_keypair
        )
        .is_ok());

        assert_eq!(
            bc.get_account_by_id(account_alice).unwrap().balance,
            99900000
        );
        assert_eq!(bc.get_account_by_id(account_bob).unwrap().balance, 100000);
    }

    #[test]
    fn test_transfer_fail() {
        let bc = &mut Blockchain::new();

        let (account_alice, alice_keypair) = utils::generate_account_id();
        let (account_bob, bob_keypair) = utils::generate_account_id();

        assert!(utils::create_accounts_and_transfer(
            bc,
            account_alice.clone(),
            account_bob.clone(),
            account_bob.clone(),
            account_alice.clone(),
            100_000_000,
            100_000,
            &alice_keypair,
            &bob_keypair
        )
        .is_err());

        assert!(utils::create_accounts_and_transfer(
            bc,
            account_alice.clone(),
            account_bob.clone(),
            account_alice.clone(),
            account_bob.clone(),
            100_000_000,
            100_000_001,
            &alice_keypair,
            &bob_keypair
        )
        .is_err());

        let (account_bob2, _) = utils::generate_account_id();

        assert!(utils::create_accounts_and_transfer(
            bc,
            account_alice.clone(),
            account_bob.clone(),
            account_alice.clone(),
            account_bob2.clone(),
            100_000_000,
            100_000,
            &alice_keypair,
            &bob_keypair
        )
        .is_err());
    }

    #[test]
    fn test_signature() {
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

        assert!(append_block_with_tx(
            bc,
            1,
            vec![
                Transaction::new(
                    TransactionData::CreateAccount {
                        account_id: account_1.clone(),
                        public_key: account_1_keypair.public
                    },
                    None
                ),
                Transaction::new(
                    TransactionData::CreateAccount {
                        account_id: account_2.clone(),
                        public_key: account_2_keypair.public
                    },
                    None
                ),
                Transaction::new(
                    TransactionData::MintInitialSupply {
                        to: account_1.clone(),
                        amount: 100_000_000
                    },
                    None
                ),
                transfer_tx.clone()
            ]
        )
        .is_err());

        transfer_tx.sign(&account_1_keypair);

        assert!(append_block_with_tx(
            bc,
            1,
            vec![
                Transaction::new(
                    TransactionData::CreateAccount {
                        account_id: account_1.clone(),
                        public_key: account_1_keypair.public
                    },
                    None
                ),
                Transaction::new(
                    TransactionData::CreateAccount {
                        account_id: account_2.clone(),
                        public_key: account_2_keypair.public
                    },
                    None
                ),
                Transaction::new(
                    TransactionData::MintInitialSupply {
                        to: account_1.clone(),
                        amount: 100_000_000
                    },
                    None
                ),
                transfer_tx
            ]
        )
        .is_ok());
    }

    #[test]
    fn test_target() {
        let bc = &mut Blockchain::new();

        for _ in 0..12 {
            append_block(bc);
        }

        let mut prev_time = bc.blocks.head().unwrap().timestamp;

        for block in bc.blocks.iter() {
            dbg!(&prev_time - &block.timestamp);
            prev_time = block.timestamp;
        }

        for i in 0..bc.blocks.len() {
            dbg!((i, bc.get_target(i as u128)));
        }
    }
}
