use super::hash::{Hashable, H256};

/// A Merkle node that represents a node in the Merkle Tree.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MerkleNode {
    Leaf(H256),
    Parent(H256, H256),
}

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    root: Option<H256>,
    nodes: Vec<MerkleNode>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self
    where
        T: Hashable,
    {
        let leaf_nodes: Vec<MerkleNode> = data.iter().map(|d| MerkleNode::Leaf(d.hash())).collect();
        let mut nodes = leaf_nodes.clone();
        let mut current_level = leaf_nodes;

        while current_level.len() > 1 {
            let mut new_level = vec![];

            for chunk in current_level.chunks(2) {
                let left = &chunk[0];
                let right = &chunk[1];
                let parent_hash = H256::combine(left.hash(), right.hash());
                new_level.push(MerkleNode::Parent(left.hash(), right.hash()));
                nodes.push(MerkleNode::Parent(left.hash(), right.hash()));
                nodes.push(parent_hash);
            }

            current_level = new_level;
        }

        let root = current_level[0].clone();

        MerkleTree {
            root: Some(match &root {
                MerkleNode::Leaf(h) => h.clone(),
                MerkleNode::Parent(_, _) => root.hash(),
            }),
            nodes,
        }
    }

    pub fn root(&self) -> H256 {
        self.root.expect("Merkle Tree is empty")
    }

    pub fn proof(&self, index: usize) -> Vec<H256> {
        if let Some(node) = self.nodes.get(index) {
            let mut proof = vec![];
            let mut current_index = index;

            while current_index > 0 {
                let sibling_index = if current_index % 2 == 0 {
                    current_index - 1
                } else {
                    current_index + 1
                };

                if let Some(sibling) = self.nodes.get(sibling_index) {
                    proof.push(match sibling {
                        MerkleNode::Leaf(h) => h.clone(),
                        MerkleNode::Parent(_, _) => sibling.hash(),
                    });
                } else {
                    break;
                }

                current_index = (current_index - 1) / 2;
            }

            proof
        } else {
            vec![]
        }
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize) -> bool {
    let mut current_hash = datum.clone();

    for sibling_hash in proof {
        if index % 2 == 0 {
            current_hash = H256::combine(current_hash, sibling_hash.clone());
        } else {
            current_hash = H256::combine(sibling_hash.clone(), current_hash);
        }
        index /= 2;
    }

    &current_hash == root
}

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
            (hex!(
                "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920"
            ))
            .into()
        );
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(
            proof,
            vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0));
    }
}
