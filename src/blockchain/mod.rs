use crate::types::block::Block;
use crate::types::hash::H256;

use std::collections::HashMap;

pub struct BlockNode {
    block: Block, 
    height: u64
}

pub struct Blockchain {
    map: HashMap<H256, BlockNode>,
    tip: H256
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut map = HashMap::new();

        let genesis_parent: H256 = (hex!("0000000000000000000000000000000000000000000000000000000000000000")).into();
        let nonce: u32 = 0;
        
        let transactions: Vec<SignedTransaction> = Vec::new();
        let content = Content{transactions};
        let merkle_tree = MerkleTree::new(&transactions);
        let merkle_root = merkle_tree.root();
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        unimplemented!()
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        unimplemented!()
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        // unimplemented!()
        vec![]
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