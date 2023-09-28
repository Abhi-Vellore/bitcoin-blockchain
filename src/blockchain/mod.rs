use crate::types::{
    block::{Block, Content, Header},
    hash::{H256, Hashable},
    transaction::SignedTransaction,
    merkle::MerkleTree,
};
use std::{
    collections::HashMap,
    time::SystemTime,
};

// A BlockNode is a node in the Blockchain
pub struct BlockNode {
    block: Block, 
    height: u64
}

// A Blockchain
pub struct Blockchain {
    map: HashMap<H256, BlockNode>,
    tip: H256
}

// Implement functions for the Blockchain
impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut map = HashMap::new();

        let genesis_parent: H256 = (hex!("0000000000000000000000000000000000000000000000000000000000000000")).into();
        let nonce: u32 = 0;
        
        let transactions: Vec<SignedTransaction> = Vec::new();
        let merkle_tree = MerkleTree::new(&transactions);
        let merkle_root = merkle_tree.root();
        
        let difficulty: H256 = (hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")).into();
        let timestamp: SystemTime = SystemTime::now();

        let content = Content{ transactions };

        let header = Header {
            parent: genesis_parent,
            nonce: nonce,
            difficulty: difficulty,
            timestamp: timestamp,
            merkle_root: merkle_root
        };

        let genesis_block = Block{ header, content };
        let tip = genesis_block.hash();

        map.insert(genesis_block.hash(), BlockNode{ block: genesis_block, height: 0 });

        Blockchain{ map, tip }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let parent_node = self.map.get(&block.get_parent()).unwrap();
        let height = parent_node.height + 1;

        let blocknode = BlockNode { 
            block: block.clone(), 
            height: height 
        }; 

        // Insert blocknode into hashmap
        self.map.insert(block.hash(), blocknode);

        // Update tip
        let tip_node = self.map.get(&self.tip).unwrap();        
        if height > tip_node.height {
            self.tip = block.hash();
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip;
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut longest_chain: Vec<H256> = Vec::new();
        let mut cur_block_hash: H256 = self.tip;

        loop {
            longest_chain.push(cur_block_hash);

            let blocknode = self.map.get(&cur_block_hash).unwrap();

            if blocknode.height == 0 {
                break;
            }

            cur_block_hash = blocknode.block.get_parent();
        }

        return longest_chain.into_iter().rev().collect();
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST