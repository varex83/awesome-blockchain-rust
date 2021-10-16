use crate::types::Balance;

#[derive(Debug)]
pub enum AccountType {
    User,
    Contract,
}

#[derive(Debug)]
pub struct Account {
    account_type: AccountType,
    balance: Balance,
}

impl Account {
    pub fn new(account_type: AccountType) -> Self {
        Self {
            account_type,
            balance: 0,
        }
    }
}
