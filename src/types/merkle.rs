use super::hash::{Hashable, H256};
use ring::digest::{Context, SHA256};


/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    root: Option<H256>,
    nodes: Vec<Option<H256>>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        if data.is_empty() {
            return MerkleTree {
                root: None,
                nodes: vec![],
            };
        }

        let base: i32 = 2; // for exponentials

        let mut leaf_count = data.len();
        let tree_size = 2 * leaf_count.next_power_of_two() - 1;
        let mut nodes = vec![None; tree_size];

        let mut level = ((leaf_count.next_power_of_two()) as f32).log2() as i32;
        // Fill in the leaf nodes with hashed data
        for (i, item) in data.iter().enumerate() {
            nodes[base.pow(level as u32) as usize - 1 + i] = Some(item.hash());
        }

        // add duplicate node to leaf row
        if(leaf_count % 2 == 1) {
            nodes[base.pow(level as u32) as usize + leaf_count-1] = nodes[base.pow(level as u32) as usize + leaf_count-2];
            leaf_count = leaf_count + 1;
        }

    
        let mut level_count = leaf_count / 2;

        for lvl in (0..level).rev() {
            for i in (0..level_count) {
                let left = nodes[2 * i + 1].clone().unwrap_or_default();
                let right = nodes[2 * i + 2].clone().unwrap_or_default();
                //let combined_hash = H256::combine(&left, &right);

                // create and update a context with both left and right hashes
                let mut context = Context::new(&SHA256);
                let left_bytes: &[u8] = left.as_ref();
                let right_bytes: &[u8] = right.as_ref();
                context.update(&left_bytes);
                context.update(&right_bytes);

                // Finish the context to compute the combined hash
                let combined_hash = context.finish();
                let combined_hash_bytes = combined_hash.as_ref().to_owned();

                let mut combined_hash_array: [u8; 32] = [0; 32];
                combined_hash_array.copy_from_slice(&combined_hash_bytes);

                let h256: H256 = H256::from(&combined_hash_array);
                
                
                nodes[base.pow(lvl as u32) as usize - 1 + i] = Some(h256);
            }
            
            // add duplicate to end of row if necessary
            if (level_count % 2 == 1) {
                nodes[base.pow(lvl as u32) as usize + level_count - 1] = nodes[base.pow(lvl as u32) as usize + level_count - 2];
            }

            // update level count
            level_count = leaf_count / 2;
        }

        MerkleTree {
            root: nodes[0].clone(),
            nodes,
        }

    }

    pub fn root(&self) -> H256 {
        self.root.unwrap()
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        if index >= self.nodes.len() {
            // Return an empty vector if the index is out of bounds
            return Vec::new();
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        let max_level = ((self.nodes.len().next_power_of_two()) as f32).log2() as i32;

        // Start from the leaf level and go up to the parent level (excluding root)
        for level in (1..max_level).rev() {
            if current_index % 2 == 0 {
                // If the current node is a right child, add the sibling on the left
                let sibling_index = current_index - 1;
                let sibling_hash = &self.nodes[sibling_index];
                proof.push(sibling_hash.unwrap());
            } else {
                // If the current node is a left child, add the sibling on the right
                let sibling_index = current_index + 1;
                let sibling_hash = &self.nodes[sibling_index];
                proof.push(sibling_hash.unwrap());
            }
        }
        proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
// pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
//     if index >= leaf_size {
//         return false; // Index out of bounds
//     }

//     let mut computed_hash = datum.clone();

//     for (i, proof_hash) in proof.iter().enumerate() {
//         let sibling_index = if index % 2 == 0 {
//             index + 1
//         } else {
//             index - 1
//         };

//         if sibling_index < leaf_size {
//             if i % 2 == 0 {
//                 computed_hash = MerkleTree::combine_hashes(proof_hash, &computed_hash);
//             } else {
//                 computed_hash = MerkleTree::combine_hashes(&computed_hash, proof_hash);
//             }

//             index /= 2;
//         } else {
//             // Invalid proof
//             return false;
//         }
//     }
//     &computed_hash == root
// }
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    // #[test]
    // fn merkle_verifying() {
    //     let input_data: Vec<H256> = gen_merkle_tree_data!();
    //     let merkle_tree = MerkleTree::new(&input_data);
    //     let proof = merkle_tree.proof(0);
    //     assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    // }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST