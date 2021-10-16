use blake2::{Blake2s, Digest};
use blockchain_workshop::traits::Hashable;
use blockchain_workshop::types::{Transaction, TransactionData};
use ed25519_dalek::{Keypair, Signature, Signer, Verifier};
use std::ops::Add;

fn main() {
    let keypair_bob = Keypair::generate(&mut rand::rngs::OsRng {});
    let mut tx = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 100,
        },
        Some("bob".to_string()),
    );
    let pub_key_bob = keypair_bob.public;

    let signature_bytes = keypair_bob.sign(tx.hash().as_bytes()).to_bytes();

    // Blockchain

    let mut tx_invalid = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 1,
        },
        Some("bob".to_string()),
    );

    dbg!(pub_key_bob
        .verify(
            tx_invalid.hash().as_bytes(),
            &Signature::from(signature_bytes)
        )
        .is_ok());

    dbg!(pub_key_bob
        .verify(tx.hash().as_bytes(), &Signature::from(signature_bytes))
        .is_ok());
}
