use crate::types::{AccountId, Balance, Timestamp};

pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    from: AccountId,
    data: TransactionData,
    signature: Option<String>,
}

pub enum TransactionData {
    CreateAccount(AccountId),
    MintInitialSupply { to: AccountId, amount: Balance },
    Transfer { to: AccountId, amount: Balance },
}
