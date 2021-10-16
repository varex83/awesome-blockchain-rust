use crate::traits::Hashable;
use crate::types::{Account, AccountId, Block, Chain, Error, Hash, Transaction};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Blockchain {
    blocks: Chain<Block>,
    accounts: HashMap<AccountId, Account>,
    transaction_pool: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), Error> {
        self.blocks.append(block);
        Ok(())
    }

    pub fn get_last_block_hash(&self) -> Option<Hash> {
        self.blocks.head().map(|block| block.hash())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bc = Blockchain::new();
        assert_eq!(bc.get_last_block_hash(), None);
    }

    #[test]
    fn test_append() {
        let mut bc = Blockchain::new();
        let mut block = Block::new(None);
        block.set_nonce(1);

        bc.append_block(block.clone());

        block.set_nonce(2);
        bc.append_block(block.clone());

        let hash = block.hash;

        assert_eq!(bc.get_last_block_hash(), hash);
    }
}
