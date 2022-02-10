mod account;
mod block;
mod blockchain;
mod chain;
mod transaction;
pub(crate) mod miner;

pub use account::{Account, AccountType};
pub use block::Block;
pub use blockchain::Blockchain;
pub use chain::Chain;
pub use transaction::{Transaction, TransactionData};
pub use miner::{mine};

pub type Hash = String;
pub type Timestamp = u128;
pub type Balance = u128;
pub type Error = String;
pub type AccountId = String;
