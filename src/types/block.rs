use serde::{Serialize, Deserialize};
use crate::types::hash::{H256, Hashable};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Header {
    parent: H256,
    nonce: u32,
    difficulty: H256,
    timestamp: u64,
    merkle_root: H256,
}

// Implement Hashable trait for Header
impl Hashable for Header {
    fn hash(&self) -> H256 {
        // Implement the hashing logic for a block header
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.parent);
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes.extend_from_slice(&self.difficulty);
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.merkle_root);
        H256::hash(&bytes)
    }
}

// Define the structure for block content (transactions)
pub struct Content {
    transactions: Vec<SignedTransaction>,
}


pub struct Block {
    header: Header,
    content: Content,
}


impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {

    pub fn new(parent: H256, content: Content) -> Self {
        let nonce = 0; // You should generate a random nonce
        let difficulty = H256::default(); 
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let merkle_root = content.calculate_merkle_root();

        Block {
            header: Header {
                parent,
                nonce,
                difficulty,
                timestamp,
                merkle_root,
            },
            content,
        }
    }

    pub fn get_parent(&self) -> H256 {
        &self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        &self.header.difficulty
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    let content = Content {
        transactions: vec![],
    };

    Block::new(parent, content)
}