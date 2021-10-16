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
}

impl Account {
    pub fn new(account_type: AccountType) -> Self {
        Self {
            account_type,
            balance: 0,
        }
    }
}
