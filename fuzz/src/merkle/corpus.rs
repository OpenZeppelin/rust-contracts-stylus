use std::{fs, io::Write, path::Path};

use rs_merkle::Hasher;
use test_fuzz::CommutativeKeccak256;

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

    // Create test cases
    let test_cases = vec![
        // Case 1: Basic tree with 3 leaves
        TestCase {
            name: "basic_3_leaves",
            leaves: vec![hash_leaf(b"a"), hash_leaf(b"b"), hash_leaf(b"c")],
            indices_to_prove: vec![1], // Prove leaf "b"
            proof_flags: vec![false, false], // Simple proof flags
        },
        // Case 2: More complex tree with 7 leaves (perfect binary tree)
        TestCase {
            name: "perfect_tree_7_leaves",
            leaves: vec![
                hash_leaf(b"data1"),
                hash_leaf(b"data2"),
                hash_leaf(b"data3"),
                hash_leaf(b"data4"),
                hash_leaf(b"data5"),
                hash_leaf(b"data6"),
                hash_leaf(b"data7"),
            ],
            indices_to_prove: vec![2, 5], // Prove leaves "data3" and "data6"
            proof_flags: vec![false, true, false, true], // Multiple proof flags
        },
        // Case 3: Edge case with exactly 2 leaves
        TestCase {
            name: "minimal_2_leaves",
            leaves: vec![hash_leaf(b"left"), hash_leaf(b"right")],
            indices_to_prove: vec![0], // Prove "left" leaf
            proof_flags: vec![false],  // Minimal flags
        },
        // Case 4: Edge case with zero-value leaf
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
        // Case 5: Maximum number of leaves in our test range
        TestCase {
            name: "max_leaves",
            leaves: (0..30).map(|i| hash_leaf(&[i as u8])).collect(),
            indices_to_prove: vec![5, 15, 25], // Prove multiple leaves
            proof_flags: vec![false, true, false, true, false, true, false],
        },
        // Case 6: Testing with duplicate data (different indices)
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
        // Case 7: Special case testing with specific bit patterns
        TestCase {
            name: "bit_patterns",
            leaves: vec![
                [0xffu8; 32], // All bits set
                [0x00u8; 32], // No bits set
                [0xaau8; 32], // Alternating bits
                [0x55u8; 32], // Alternating bits (inverted)
            ],
            indices_to_prove: vec![0, 1], // Prove both extremes
            proof_flags: vec![false, true, false],
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
