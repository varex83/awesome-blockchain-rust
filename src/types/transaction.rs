use crate::traits::{Hashable, WorldState};
use crate::types::{AccountId, AccountType, Balance, Error, Hash, Timestamp};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};
use ed25519_dalek::{Keypair, Signature, Signer, Verifier};

#[derive(Debug, Clone)]
pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    from: Option<AccountId>,
    pub(crate) data: TransactionData,
    signature: Option<Signature>,
}

#[derive(Debug, Clone)]
pub enum TransactionData {
    CreateAccount{ account_id: AccountId, public_key: ed25519_dalek::PublicKey },
    MintInitialSupply { to: AccountId, amount: Balance },
    Transfer { to: AccountId, amount: Balance },
}

impl Transaction {
    pub fn new(data: TransactionData, from: Option<AccountId>) -> Self {
        Self {
            nonce: 0,
            timestamp: 0,
            from,
            data,
            signature: None,
        }
    }

    pub fn verify_signature<T: WorldState>(&self, state: &mut T) -> Result<(), Error> {
        if !self.signature.is_some() {
            return Err("Error: msg should be signed".to_string());
        }

        if !self.from.is_some() {
            return Err("Error: msg should have sender to sign it".to_string());
        }

        let account = self.from.clone().unwrap().clone();

        let account = state.get_account_by_id(account);

        if account.is_none() {
            return Err("Error: msg should have sender to sign it".to_string());
        }

        let account = account.unwrap();

        match account.public_key.verify(
            self.hash().as_bytes(),
            &self.signature.unwrap()
        ) {
            Ok(()) => Ok(()),
            Err(_) => Err("Error: error occurred while verifying signature".to_string())
        }
    }

    pub fn sign(&mut self, keypair: &Keypair) {
        self.signature = Some(keypair.sign(self.hash().as_bytes()));
    }

    pub fn execute<T: WorldState>(&self, state: &mut T, is_genesis: bool) -> Result<(), Error> {
        //TODO Task 2: Implement signature
        match &self.data {
            TransactionData::CreateAccount {account_id, public_key}  => {
                state.create_account(
                    account_id.clone(),
                    AccountType::User,
                    *public_key
                )
            }
            TransactionData::MintInitialSupply { to, amount } => {
                if !is_genesis {
                    return Err("Initial supply can be minted only in genesis block.".to_string());
                }
                match state.get_account_by_id_mut(to.clone()) {
                    Some(account) => {
                        account.balance += amount;
                        Ok(())
                    }
                    None => Err("Invalid account.".to_string()),
                }
            }
            // DONE Task 1: Implement transfer transition function
            // 1. Check that receiver and sender accounts exist
            // 2. Check sender balance
            // 3. Change sender/receiver balances and save to state
            // 4. Test
            TransactionData::Transfer { to, amount } => {
                let from = self.from.clone();

                if !from.is_some() {
                    return Err("You can't make transfer from non-existing account".to_string())
                }

                let from = from.unwrap();

                if !state.get_account_by_id_mut(from.clone()).is_some() {
                    return Err("You can't make transfer from non-existing account".to_string())
                }
                if !state.get_account_by_id_mut(to.clone()).is_some() {
                    return Err("You can't make transfer to non-existing account".to_string())
                };

                if let Err(e) = self.verify_signature(state) {
                    return Err(format!("Error while verifying signature: {}", e));
                }

                let from = state.get_account_by_id_mut(from.clone()).unwrap();

                if &from.balance < amount {
                    return Err("You can't transfer more tokens than you have".to_string())
                }

                from.balance -= amount;

                let to = state.get_account_by_id_mut(to.clone()).unwrap();

                to.balance   += amount;

                Ok(())
            },
        }
    }
}

impl Hashable for Transaction {
    fn hash(&self) -> Hash {
        let mut hasher = Blake2s::new();

        hasher.update(format!(
            "{:?}",
            (
                self.nonce,
                self.timestamp,
                self.from.clone(),
                self.data.clone()
            )
        ));

        hex::encode(hasher.finalize_fixed())
    }
}
