use std::{fs, io::Write, path::Path};

use alloy_primitives::hex;
use test_fuzz::Input;

/// Simple struct to represent our test cases
struct TestCase {
    name: &'static str,
    input: Input,
}

/// Writes a binary corpus file for libFuzzer
fn write_corpus_file(dir_path: &str, case: &TestCase) -> std::io::Result<()> {
    let dir = Path::new(dir_path);
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    let file_path = dir.join(format!("seed_{}", case.name));
    let mut file = fs::File::create(file_path)?;

    // Fuzzer will reconstruct the Input struct from the data, so we need to
    // write the data in the order in which it would be listed if it were
    // flattened
    let Input { root, leaves, proof, proof_flags } = &case.input;

    // Write the root length
    file.write_all(&root.len().to_le_bytes())?;
    // Write the root
    file.write_all(root)?;

    // Write the number of leaves
    let num_leaves = leaves.len();
    file.write_all(&num_leaves.to_le_bytes())?;

    // Write each leaf
    for leaf in leaves {
        file.write_all(leaf)?;
    }

    // Write the number of multi-proof hashes
    let num_proof_hashes = proof.len();
    file.write_all(&num_proof_hashes.to_le_bytes())?;

    // Write each proof hash
    for proof_hash in proof {
        file.write_all(proof_hash)?;
    }

    // Write the number of proof flags
    let num_flags = proof_flags.len();
    file.write_all(&num_flags.to_le_bytes())?;

    // Write each flag as a byte
    for &flag in proof_flags {
        let flag_byte = if flag { 1u8 } else { 0u8 };
        file.write_all(&[flag_byte])?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let corpus_dir = "corpus/merkle";

    let test_cases = vec![
        TestCase {
            name: "3_leaves_valid",
            input: Input {
                root: hex!("6deb52b5da8fd108f79fab00341f38d2587896634c646ee52e49f845680a70c8"),
                leaves: vec![
                    hex!("19ba6c6333e0e9a15bf67523e0676e2f23eb8e574092552d5e888c64a4bb3681"),
                    hex!("c62a8cfa41edc0ef6f6ae27a2985b7d39c7fea770787d7e104696c6e81f64848"),
                    hex!("eba909cf4bb90c6922771d7f126ad0fd11dfde93f3937a196274e1ac20fd2f5b"),
                ],
                proof: vec![
                    hex!("9a4f64e953595df82d1b4f570d34c4f4f0cfaf729a61e9d60e83e579e1aa283e"),
                    hex!("8076923e76cf01a7c048400a2304c9a9c23bbbdac3a98ea3946340fdafbba34f"),
                ],
                proof_flags: vec![false, true, false, true],
            },
        },
        TestCase {
            name: "minimum_valid_case",
            input: Input {
                root: [0u8; 32],
                leaves: vec![[0u8; 32]],
                proof: vec![],
                proof_flags: vec![],
            },
        },
        TestCase {
            name: "empty_flags_valid",
            input: Input {
                root: [0u8; 32],
                leaves: vec![[1u8; 32]],
                proof: vec![[2u8; 32]],
                proof_flags: vec![],
            },
        },
        TestCase {
            name: "3_leaves_invalid",
            input: Input {
                root: hex!("6deb52b5da8fd108f79fab00341f38d2587896634c646ee52e49f845680a70c8"),
                leaves: vec![
                    hex!("34e6ce3d0d73f6bff2ee1e865833d58e283570976d70b05f45c989ef651ef742"),
                    hex!("aa28358fb75b314c899e16d7975e029d18b4457fd8fd831f2e6c17ffd17a1d7e"),
                    hex!("e0fd7e6916ff95d933525adae392a17e247819ebecc2e63202dfec7005c60560"),
                ],
                proof: vec![],
                proof_flags: vec![true, true],
            },
        },
        TestCase {
            name: "empty_leaves_valid",
            input: Input {
                root: hex!("8f7234e8cfe39c08ca84a3a3e3274f574af26fd15165fe29e09cbab742daccd9"),
                leaves: vec![],
                proof: vec![
                    hex!("8f7234e8cfe39c08ca84a3a3e3274f574af26fd15165fe29e09cbab742daccd9"), // same as root
                ],
                proof_flags: vec![],
            },
        },
        // Merkle tree that contains a zero leaf at depth 1
        //
        // Taken from https://github.com/advisories/GHSA-wprv-93r4-jj2p
        //
        // ```js
        // const { MerkleTree } = require('merkletreejs'); // v0.2.32
        // const keccak256 = require('keccak256'); // v1.0.6
        //
        // const leaves = [keccak256('real leaf'), Buffer.alloc(32, 0)];
        // const merkleTree = new MerkleTree(leaves, keccak256, { sortPairs: true });
        // const root = merkleTree.getRoot();
        // ```
        TestCase {
            name: "manipulated_multi_proof",
            input: Input {
                root: hex!("f2d552e1e4c59d4f0fa2b80859febc9e4bdc915dff37c56c858550d8b64659a5"),
                // malicious leaves
                leaves: vec![
                    hex!("1f23ad5fc0ee6ccbe2f3d30df856758f05ad9d03408a51a99c1c9f0854309db2"),
                    hex!("4e7e8301f5d206748d1c4f822e3564ddb1124f86591a839f58dfc2f007983b61"),
                    hex!("613994f4e324d0667c07857cd5d147994bc917da5d07ee63fc3f0a1fe8a18e34"),
                ],
                // [leaves[0], leaves[0]]
                proof: vec![
                    hex!("5e941ddd8f313c0b39f92562c0eca709c3d91360965d396aaef584b3fa76889a"),
                    hex!("5e941ddd8f313c0b39f92562c0eca709c3d91360965d396aaef584b3fa76889a"),
                ],
                proof_flags: vec![true, true, false],
            },
        },
        // ```js
        // const merkleTree = StandardMerkleTree.of(toElements('abcd'), ['string']);
        //
        // const root = merkleTree.root;
        // const hashA = merkleTree.leafHash(['a']);
        // const hashB = merkleTree.leafHash(['b']);
        // const hashCD = hashPair(
        //   ethers.toBeArray(merkleTree.leafHash(['c'])),
        //   ethers.toBeArray(merkleTree.leafHash(['d'])),
        // );
        // const hashE = merkleTree.leafHash(['e']); // incorrect (not part of the tree)
        // const fill = ethers.randomBytes(32);
        // ```
        TestCase {
            name: "invalid_leaf",
            input: Input {
                root: hex!("8f7234e8cfe39c08ca84a3a3e3274f574af26fd15165fe29e09cbab742daccd9"),
                // malicious leaves
                leaves: vec![
                    hex!("9a4f64e953595df82d1b4f570d34c4f4f0cfaf729a61e9d60e83e579e1aa283e"), // hashE
                    hex!("9c15a6a0eaeed500fd9eed4cbeab71f797cefcc67bfd46683e4d2e6ff7f06d1c"), // hashA
                ],
                // [hashB, fill, hashCD]
                proof: vec![
                    hex!("19ba6c6333e0e9a15bf67523e0676e2f23eb8e574092552d5e888c64a4bb3681"),
                    hex!("1111111111111111111111111111111111111111111111111111111111111111"),
                    hex!("03707d7802a71ca56a8ad8028da98c4f1dbec55b31b4a25d536b5309cc20eda9"),
                ],
                proof_flags: vec![false, false, false, false],
            },
        },
    ];

    // Write each test case to a file
    for case in test_cases {
        write_corpus_file(corpus_dir, &case)?;
        println!("Created corpus file: seed_{}", case.name);
    }

    println!("Corpus generation complete!");
    Ok(())
}
