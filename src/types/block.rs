use crate::types::{
    hash::{H256, Hashable},
    transaction::SignedTransaction,
    merkle::MerkleTree,
};

use rand::Rng;
use bincode;
use serde::{Serialize, Deserialize};
use serde::ser::{SerializeStruct, Serializer};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header,
    content: Content,
}

// A Header
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    parent: H256,
    nonce: u32,
    difficulty: H256,
    timestamp: SystemTime,
    merkle_root: H256,
}

// Implement Hashable trait for Header
impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized_header: Vec<u8> = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &serialized_header).into()
    }
}

// Implement Hashable trait for SignedTransaction
impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        // Serialize the transaction into bytes
        let serialized_transaction: Vec<u8> = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &serialized_transaction).into()
    }
}

// Define the structure for block content (transactions)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    transactions: Vec<SignedTransaction>
}

impl Content {
    pub fn calculate_merkle_root(&self) -> H256 {
        unimplemented!()
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    // Make nonce a random integer
    let mut rng = rand::thread_rng();  // create a random number generator
    let nonce: u32 = rng.gen();

    let difficulty = H256::default();
    let timestamp = SystemTime::now();

    let transactions: Vec<SignedTransaction> = Vec::new();  // empty transactions vector
    let content = Content{transactions};
    let merkle_tree = MerkleTree::new(&transactions);
    let merkle_root = merkle_tree.root();

    let header = Header {
        parent: *parent,
        nonce: nonce,
        difficulty: difficulty,
        timestamp: timestamp,
        merkle_root: merkle_root
    };

    Block{
        header: header, 
        content: content
    };
}