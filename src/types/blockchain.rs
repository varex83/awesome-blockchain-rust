use crate::types::{Account, AccountId, Block, Transaction};
use std::collections::HashMap;

pub struct Blockchain {
    blocks: Vec<Block>,
    accounts: HashMap<AccountId, Account>,
    transaction_pool: Vec<Transaction>,
}
