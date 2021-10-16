use crate::types::{Account, AccountId, AccountType, Error, Hash};

pub trait Hashable {
    fn hash(&self) -> Hash;
}

pub trait WorldState {
    fn create_account(
        &mut self,
        account_id: AccountId,
        account_type: AccountType,
    ) -> Result<(), Error>;
    fn get_account_by_id(&self, account_id: AccountId) -> Option<&Account>;
    fn get_account_by_id_mut(&mut self, account_id: AccountId) -> Option<&mut Account>;
}
