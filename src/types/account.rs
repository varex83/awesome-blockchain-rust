use crate::types::Balance;

pub enum AccountType {
    User,
    Contract,
}

pub struct Account {
    account_type: AccountType,
    balance: Balance,
}
