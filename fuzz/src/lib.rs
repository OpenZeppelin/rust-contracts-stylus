use libfuzzer_sys::arbitrary::{
    Arbitrary, Result as ArbitraryResult, Unstructured,
};

type Bytes32 = [u8; 32];

const MAX_LEAVES: usize = 64;

#[derive(Debug)]
pub struct Input {
    pub root: Bytes32,
    pub leaves: Vec<Bytes32>,
    pub proof: Vec<Bytes32>,
    pub proof_flags: Vec<bool>,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let root = u.arbitrary()?;
        let proof: Vec<Bytes32> = u.arbitrary()?;

        // leaves.len() + proof.len() >= 1
        let min_leaves = if proof.is_empty() { 1 } else { 0 };
        // ensure we don't go overboard with number of leaves
        let num_leaves = u.int_in_range(min_leaves..=MAX_LEAVES)?;
        let mut leaves = Vec::with_capacity(num_leaves);
        for _ in 0..num_leaves {
            leaves.push(u.arbitrary()?);
        }

        // ensure we pass the proof flag length check
        let proof_flag_len = leaves.len() + proof.len() - 1;
        let mut proof_flags = Vec::with_capacity(proof_flag_len);
        for _ in 0..proof_flag_len {
            proof_flags.push(u.arbitrary()?);
        }

        Ok(Input { root, leaves, proof, proof_flags })
    }
}
