use crate::traits::{Hashable, WorldState};
use crate::types::{Account, AccountId, AccountType, Block, Chain, Error, Hash, Transaction};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

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
    ) -> Result<(), Error> {
        match self.accounts.entry(account_id.clone()) {
            Entry::Occupied(_) => Err(format!("AccountId already exist: {}", account_id)),
            Entry::Vacant(v) => {
                v.insert(Account::new(account_type));
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
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), Error> {
        //TODO Task 3: Implement mining

        if !block.verify() {
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

        // TODO Task 3: Append block only if block.hash < target
        // Adjust difficulty of target each block generation (epoch)
        self.blocks.append(block);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut block_num = self.blocks.len();
        let mut prev_block_hash: Option<Hash> = None;

        for block in self.blocks.iter() {
            let is_genesis = block_num == 1;

            if !block.verify() {
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

    pub fn get_last_block_hash(&self) -> Option<Hash> {
        self.blocks.head().map(|block| block.hash())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TransactionData;
    use crate::utils::{append_block, append_block_with_tx};

    #[test]
    fn test_new() {
        let bc = Blockchain::new();
        assert_eq!(bc.get_last_block_hash(), None);
    }

    #[test]
    fn test_append() {
        let bc = &mut Blockchain::new();

        append_block(bc, 1);
        let block = append_block(bc, 2);

        assert_eq!(bc.get_last_block_hash(), block.hash);
    }

    #[test]
    fn test_create_genesis_block() {
        let bc = &mut Blockchain::new();

        let tx_create_account =
            Transaction::new(TransactionData::CreateAccount("satoshi".to_string()), None);
        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
                amount: 100_000_000,
            },
            None,
        );
        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account, tx_mint_initial_supply]).is_ok()
        );

        let satoshi = bc.get_account_by_id("satoshi".to_string());

        assert!(satoshi.is_some());
        assert_eq!(satoshi.unwrap().balance, 100_000_000);
    }

    #[test]
    fn test_create_genesis_block_fails() {
        let mut bc = Blockchain::new();

        let tx_create_account =
            Transaction::new(TransactionData::CreateAccount("satoshi".to_string()), None);
        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
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

        let tx_create_account =
            Transaction::new(TransactionData::CreateAccount("satoshi".to_string()), None);
        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
                amount: 100_000_000,
            },
            None,
        );
        let mut block = Block::new(None);
        block.set_nonce(1);
        block.add_transaction(tx_create_account);
        block.add_transaction(tx_mint_initial_supply);

        assert!(bc.append_block(block).is_ok());

        let mut block = Block::new(bc.get_last_block_hash());
        let tx_create_alice =
            Transaction::new(TransactionData::CreateAccount("alice".to_string()), None);
        let tx_create_bob =
            Transaction::new(TransactionData::CreateAccount("bob".to_string()), None);
        block.set_nonce(2);
        block.add_transaction(tx_create_alice);
        block.add_transaction(tx_create_bob.clone());
        block.add_transaction(tx_create_bob);

        assert!(bc.append_block(block).is_err());

        assert!(bc.get_account_by_id("satoshi".to_string()).is_some());
        assert!(bc.get_account_by_id("alice".to_string()).is_none());
        assert!(bc.get_account_by_id("bob".to_string()).is_none());
    }

    #[test]
    fn test_validate() {
        let bc = &mut Blockchain::new();

        let tx_create_account =
            Transaction::new(TransactionData::CreateAccount("satoshi".to_string()), None);
        let tx_mint_initial_supply = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
                amount: 100_000_000,
            },
            None,
        );
        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account, tx_mint_initial_supply]).is_ok()
        );

        append_block(bc, 2);
        append_block(bc, 3);

        assert!(bc.validate().is_ok());

        let mut iter = bc.blocks.iter_mut();
        iter.next();
        iter.next();
        let block = iter.next().unwrap();
        block.transactions[1].data = TransactionData::MintInitialSupply {
            to: "satoshi".to_string(),
            amount: 100,
        };

        assert!(bc.validate().is_err());
    }
}
