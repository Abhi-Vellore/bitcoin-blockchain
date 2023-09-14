use serde::{Serialize,Deserialize};
use ring::{
    signature::{Ed25519KeyPair, Signature, KeyPair, UnparsedPublicKey},
    agreement:: PublicKey
};
use rand::Rng;
use super::address::Address;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    sender : Address,
    receiver : Address,
    value: i32
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    transaction: Transaction,
    signature: Vec<u8>, // = Signature.as_ref().to_vec(),
    public_key: Vec<u8> // = public_key.as_ref().to_vec()
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    // Serialize the transaction
    let transaction_bytes: Vec<u8> = bincode::serialize(&t).unwrap();

    //let key_pair: &Ed25519KeyPair = key;
    //let private_key = (*key_pair).private_key();

    // Get the private key from the Ed25519KeyPair
    //let private_key = key.private_key();

    // Sign the serialized transaction with the private key
    let signature = key.sign(&transaction_bytes);

    signature
    
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    
    // Convert the transaction to a byte representation
    let transaction_bytes = bincode::serialize(&t).unwrap();

    // Create a PublicKey from the public key bytes
    let public_key = UnparsedPublicKey::new(&ring::signature::ED25519, public_key);

    // Verify the signature using the public key
    match public_key.verify(&transaction_bytes, &signature) {
        Ok(_) => true,  // Signature is valid
        Err(_) => false, // Signature is invalid
    }    
    
}
    


#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    // Create a random number generator
    
    fn generate_random_bytes() -> [u8; 20] {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 20];
        rng.fill(&mut bytes);
        bytes
    }
    let mut rng = rand::thread_rng();


    // Generate random values for sender, receiver, and value
    let sender = Address::from_public_key_bytes(&generate_random_bytes());       // Random sender address
    let receiver = Address::from_public_key_bytes(&generate_random_bytes());     // Random receiver address
    let value = rng.gen::<i32>();          // Random integer value

    // Create a new Transaction with the generated values
    Transaction {
        sender,
        receiver,
        value,
    }
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