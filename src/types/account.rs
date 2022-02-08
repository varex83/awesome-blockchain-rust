use crate::types::Balance;

#[derive(Debug, Clone)]
pub enum AccountType {
    User,
    Contract,
}

#[derive(Debug, Clone)]
pub struct Account {
    account_type: AccountType,
    pub(crate) balance: Balance,
    pub(crate) public_key: ed25519_dalek::PublicKey
}

impl Account {
    pub fn new(account_type: AccountType, public_key: ed25519_dalek::PublicKey) -> Self {
        Self {
            account_type,
            balance: 0,
            public_key
        }
    }
}
