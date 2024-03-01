use alloy_primitives::B256;
use alloy_sol_types::SolValue;

pub trait Hasher {
    type Hash: Copy + From<B256>;

    fn hash(&mut self, data: &[u8]) -> Self::Hash;
}

pub fn verify<H: Hasher<Hash = B256>>(
    proof: &[B256],
    root: B256,
    mut leaf: B256,
    mut hasher: H,
) -> bool {
    for i in 0..proof.len() {
        leaf = sorted_hash(leaf, proof[i], &mut hasher);
    }

    leaf == root
}

fn sorted_hash<H: Hasher<Hash = B256>>(mut a: B256, mut b: B256, hasher: &mut H) -> B256 {
    if a >= b {
        (a, b) = (b, a);
    }

    hasher.hash(&[a, b].abi_encode())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{keccak256, B256};
    use const_hex::FromHex;

    use crate::merkle::sorted_hash;

    use super::{verify, Hasher};

    struct Keccak256;
    impl Hasher for Keccak256 {
        type Hash = B256;

        fn hash(&mut self, data: &[u8]) -> Self::Hash {
            keccak256(data)
        }
    }

    #[test]
    fn verifies_valid_proofs() {
        // These values are generated using https://github.com/OpenZeppelin/merkle-tree.
        // They correspond to:
        //
        // ```js
        // const merkleTree = StandardMerkleTree.of(
        //   toElements('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='),
        //   ['string'],
        // );
        //
        // const root = merkleTree.root;
        // const hash = merkleTree.leafHash(['A']);
        // const proof = merkleTree.getProof(['A']);
        // ```
        const ROOT: &str = "0xb89eb120147840e813a77109b44063488a346b4ca15686185cf314320560d3f3";
        const LEAF_A: &str = "0x6efbf77e320741a027b50f02224545461f97cd83762d5fbfeb894b9eb3287c16";
        const LEAF_B: &str = "0x7051e21dd45e25ed8c605a53da6f77de151dcbf47b0e3ced3c5d8b61f4a13dbc";
        const PROOF: &str = r"0x7051e21dd45e25ed8c605a53da6f77de151dcbf47b0e3ced3c5d8b61f4a13dbc
                              0x1629d3b5b09b30449d258e35bbd09dd5e8a3abb91425ef810dc27eef995f7490
                              0x633d21baee4bbe5ed5c51ac0c68f7946b8f28d2937f0ca7ef5e1ea9dbda52e7a
                              0x8a65d3006581737a3bab46d9e4775dbc1821b1ea813d350a13fcd4f15a8942ec
                              0xd6c3f3e36cd23ba32443f6a687ecea44ebfe2b8759a62cccf7759ec1fb563c76
                              0x276141cd72b9b81c67f7182ff8a550b76eb96de9248a3ec027ac048c79649115";

        let root = B256::from_hex(ROOT).unwrap();
        let leaf_a = B256::from_hex(LEAF_A).unwrap();
        let leaf_b = B256::from_hex(LEAF_B).unwrap();
        let proof: Vec<_> = PROOF
            .lines()
            .map(|h| B256::from_hex(h.trim()).unwrap())
            .collect();

        let valid = verify(&proof, root, leaf_a, Keccak256);
        assert!(valid);

        let mut hasher = Keccak256;
        let no_such_leaf = sorted_hash(leaf_a, leaf_b, &mut hasher);
        let proof: Vec<_> = proof.into_iter().skip(1).collect();
        let valid = verify(&proof, root, no_such_leaf, hasher);
        assert!(valid);
    }

    #[test]
    fn rejects_invalid_proofs() {
        // These values are generated using https://github.com/OpenZeppelin/merkle-tree.
        // They correspond to:
        //
        // ```js
        // const correctMerkleTree = StandardMerkleTree.of(toElements('abc'), ['string']);
        // const otherMerkleTree = StandardMerkleTree.of(toElements('def'), ['string']);
        //
        // const root = correctMerkleTree.root;
        // const leaf = correctMerkleTree.leafHash(['a']);
        // const proof = otherMerkleTree.getProof(['d']);
        // ```
        const ROOT: &str = "0xf2129b5a697531ef818f644564a6552b35c549722385bc52aa7fe46c0b5f46b1";
        const LEAF: &str = "0x9c15a6a0eaeed500fd9eed4cbeab71f797cefcc67bfd46683e4d2e6ff7f06d1c";
        const PROOF: &str = "0x7b0c6cd04b82bfc0e250030a5d2690c52585e0cc6a4f3bc7909d7723b0236ece";

        let root = B256::from_hex(ROOT).unwrap();
        let leaf = B256::from_hex(LEAF).unwrap();
        let proof = B256::from_hex(PROOF).unwrap();

        let valid = verify(&[proof], root, leaf, Keccak256);
        assert!(!valid);
    }

    #[test]
    fn rejects_proofs_with_invalid_length() {
        // These values are generated using https://github.com/OpenZeppelin/merkle-tree.
        // const merkleTree = StandardMerkleTree.of(toElements('abc'), ['string']);
        //
        // const root = merkleTree.root;
        // const leaf = merkleTree.leafHash(['a']);
        // const proof = merkleTree.getProof(['a']);
        const ROOT: &str = "0xf2129b5a697531ef818f644564a6552b35c549722385bc52aa7fe46c0b5f46b1";
        const LEAF: &str = "0x9c15a6a0eaeed500fd9eed4cbeab71f797cefcc67bfd46683e4d2e6ff7f06d1c";
        const PROOF: &str = r"0x19ba6c6333e0e9a15bf67523e0676e2f23eb8e574092552d5e888c64a4bb3681
                              0x9cf5a63718145ba968a01c1d557020181c5b252f665cf7386d370eddb176517b";

        let root = B256::from_hex(ROOT).unwrap();
        let leaf = B256::from_hex(LEAF).unwrap();
        let proof: Vec<_> = PROOF
            .lines()
            .map(|h| B256::from_hex(h.trim()).unwrap())
            .collect();

        let bad_proof: Vec<_> = proof.into_iter().take(1).collect();
        let valid = verify(&bad_proof, root, leaf, Keccak256);
        assert!(!valid);
    }
}
