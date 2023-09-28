use crate::types::{
    hash::{H256, Hashable},
    transaction::SignedTransaction
};

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
    transactions: Vec<SignedTransaction>,
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
    // Create a new Block, given a parent and content
    pub fn new(parent: &H256, content: Content) -> Self {
        let nonce = 0; // You should generate a random nonce
        let difficulty = H256::default(); 
        let timestamp = SystemTime::now();
        let merkle_root = content.calculate_merkle_root();

        Block {
            header: Header {
                parent: *parent,
                nonce,
                difficulty,
                timestamp,
                merkle_root,
            },
            content,
        }
    }

    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    let content = Content {
        transactions: vec![],
    };

    Block::new(parent, content)
}