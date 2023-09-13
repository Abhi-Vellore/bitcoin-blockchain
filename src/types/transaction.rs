use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature};
use rand::Rng;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    sender : Address,
    receiever : Address,
    value: i32
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    let transaction: Transaction = Transaction

    let signature: Vec<u8> = Signature.as_ref().to_vec()
    let public_key: Vec<u8> = public_key.as_ref().to_vec()

}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    // Serialize the transaction
    let transaction_bytes: Vec<u8> = bincode.serialize(&t).unwrap();

    // Get the private key from the Ed25519KeyPair
    let private_key = key_pair.private_key();

    // Sign the serialized transaction with the private key
    let signature = private_key.sign(&transaction_bytes);

    signature
    
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    
    // Deserialize the provided public key
    let public_key = PublicKey::from_bytes(public_key);

        // Serialize the transaction to bytes
        let t_bytes = bincode::serialize(t).unwrap();

        // Deserialize the provided signature
        let signature = Signature::from_bytes(signature);

        if public_key.verify(&t_bytes, &signature) {
            return true;
        }
}


#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    unimplemented!()
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST