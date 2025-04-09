use std::{fs, io::Write, path::Path};

use rs_merkle::Hasher;
use test_fuzz::{consts::merkle::MAX_LEAVES, CommutativeKeccak256};

/// Simple struct to represent our test cases
struct TestCase {
    name: &'static str,
    leaves: Vec<[u8; 32]>,
    indices_to_prove: Vec<usize>,
    proof_flags: Vec<bool>,
}

/// Function that hashes a byte array using Keccak256
fn hash_leaf(data: &[u8]) -> [u8; 32] {
    CommutativeKeccak256::hash(data)
}

/// Writes a binary corpus file for libFuzzer
fn write_corpus_file(dir_path: &str, case: &TestCase) -> std::io::Result<()> {
    let dir = Path::new(dir_path);
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    let file_path = dir.join(format!("seed_{}", case.name));
    let mut file = fs::File::create(file_path)?;

    // First, write the number of leaves
    let num_leaves = case.leaves.len() as u32;
    file.write_all(&num_leaves.to_le_bytes())?;

    // Write each leaf
    for leaf in &case.leaves {
        file.write_all(leaf)?;
    }

    // Write the number of indices
    let num_indices = case.indices_to_prove.len() as u32;
    file.write_all(&num_indices.to_le_bytes())?;

    // Write each index
    for &idx in &case.indices_to_prove {
        let idx_u32 = idx as u32;
        file.write_all(&idx_u32.to_le_bytes())?;
    }

    // Write the number of flags
    let num_flags = case.proof_flags.len() as u32;
    file.write_all(&num_flags.to_le_bytes())?;

    // Write each flag as a byte
    for &flag in &case.proof_flags {
        let flag_byte = if flag { 1u8 } else { 0u8 };
        file.write_all(&[flag_byte])?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let corpus_dir = "corpus/merkle";

    let test_cases = vec![
        TestCase {
            name: "3_leaves_single_index",
            leaves: vec![hash_leaf(b"a"), hash_leaf(b"b"), hash_leaf(b"c")],
            indices_to_prove: vec![1],
            proof_flags: vec![false, false],
        },
        TestCase {
            name: "3_leaves_multiple_indices",
            leaves: vec![hash_leaf(b"a"), hash_leaf(b"b"), hash_leaf(b"c")],
            indices_to_prove: vec![1, 2],
            proof_flags: vec![false, true],
        },
        TestCase {
            name: "perfect_tree_4_leaves",
            leaves: vec![
                hash_leaf(b"a"),
                hash_leaf(b"b"),
                hash_leaf(b"c"),
                hash_leaf(b"d"),
            ],
            indices_to_prove: vec![0, 3],
            proof_flags: vec![false, false, true],
        },
        TestCase {
            name: "minimal_2_leaves",
            leaves: vec![hash_leaf(b"left"), hash_leaf(b"right")],
            indices_to_prove: vec![0],
            proof_flags: vec![false],
        },
        TestCase {
            name: "zero_value_leaf",
            leaves: vec![
                hash_leaf(b"normal"),
                [0u8; 32], // Zero leaf
                hash_leaf(b"another"),
            ],
            indices_to_prove: vec![1], // Prove the zero leaf
            proof_flags: vec![false, false],
        },
        TestCase {
            name: "max_leaves",
            leaves: (0..MAX_LEAVES).map(|i| hash_leaf(&[i as u8])).collect(),
            indices_to_prove: vec![5, 10, 15, 20, 25],
            proof_flags: vec![
                false, true, false, true, false, true, false, false, true,
                false, true, false, true, false, true, true, false,
            ],
        },
        TestCase {
            name: "duplicate_leaves",
            leaves: vec![
                hash_leaf(b"same"),
                hash_leaf(b"same"),
                hash_leaf(b"same"),
                hash_leaf(b"different"),
            ],
            indices_to_prove: vec![0, 2], // Prove two identical leaves
            proof_flags: vec![false, true, false],
        },
        // Special case testing with specific bit patterns
        TestCase {
            name: "bit_patterns",
            leaves: vec![
                [0xffu8; 32], // All bits set
                [0x00u8; 32], // No bits set
                [0xaau8; 32], // Alternating bits
                [0x55u8; 32], // Alternating bits (inverted)
            ],
            indices_to_prove: vec![0, 1], // Prove both extremes
            proof_flags: vec![false, true],
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
