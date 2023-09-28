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
        
        // start with the tip 
        let mut cur_block_hash: H256 = self.tip; 

        // move upwards through chain until genesis block is reached
        loop {
            longest_chain.push(cur_block_hash); 
            let cur_blocknode = self.map.get(&cur_block_hash).unwrap();
            if cur_blocknode.height == 0 { break; }   // end loop at genesis block
            cur_block_hash = cur_blocknode.block.get_parent();   // move to parent
        }

        longest_chain.reverse();   // reverses longest_chain vector in-place
        
        longest_chain
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

    #[test]
    fn insert_three() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block1 = generate_random_block(&genesis_hash);
        let block2 = generate_random_block(&block1.hash());
        let block3 = generate_random_block(&block2.hash());

        blockchain.insert(&block1);
        blockchain.insert(&block2);
        blockchain.insert(&block3);
        
        let chain = blockchain.all_blocks_in_longest_chain();

        // longest chain: gen -> b1 -> b2 -> b3
        // tip: b3
        
        assert_eq!(blockchain.tip(), block3.hash());
        assert_eq!(chain.len(), 4);
        assert_eq!(chain[0], genesis_hash);
        assert_eq!(chain[1], block1.hash());
        assert_eq!(chain[2], block2.hash());
        assert_eq!(chain[3], block3.hash());
    }

    #[test]
    fn insert_four_with_fork() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block1 = generate_random_block(&genesis_hash);
        let block2 = generate_random_block(&block1.hash());
        let block3 = generate_random_block(&block1.hash());
        let block4 = generate_random_block(&block3.hash());
        
        blockchain.insert(&block1);
        blockchain.insert(&block2);
        blockchain.insert(&block3);
        blockchain.insert(&block4);
        
        let chain = blockchain.all_blocks_in_longest_chain();

        // longest chain: gen -> b1 -> b3 -> b4
        // tip: b4
        
        assert_eq!(blockchain.tip(), block4.hash());
        assert_eq!(chain.len(), 4);
        assert_eq!(chain[0], genesis_hash);
        assert_eq!(chain[1], block1.hash());
        assert_eq!(chain[2], block3.hash());
        assert_eq!(chain[3], block4.hash());
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST