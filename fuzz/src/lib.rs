use libfuzzer_sys::arbitrary::{
    Arbitrary, Result as ArbitraryResult, Unstructured,
};

type Bytes32 = [u8; 32];

#[derive(Debug)]
pub struct SingleProof {
    pub leaf: Bytes32,
    pub proof: Vec<Bytes32>,
}

#[derive(Debug)]
pub struct MultiProof {
    pub leaves: Vec<Bytes32>,
    pub proof: Vec<Bytes32>,
    pub proof_flags: Vec<bool>,
}

#[derive(Debug)]
pub struct Input {
    pub root: Bytes32,
    pub single_proof: SingleProof,
    pub multi_proof: MultiProof,
}

impl<'a> Arbitrary<'a> for SingleProof {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let leaf = u.arbitrary()?;
        let proof: Vec<Bytes32> = u.arbitrary()?;
        Ok(SingleProof { leaf, proof })
    }
}

impl<'a> Arbitrary<'a> for MultiProof {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let leaves: Vec<Bytes32> = u.arbitrary()?;
        let proof: Vec<Bytes32> = u.arbitrary()?;

        let proof_flag_len = leaves.len() + proof.len() - 1;
        let mut proof_flags = Vec::with_capacity(proof_flag_len);
        for _ in 0..proof_flag_len {
            proof_flags.push(u.arbitrary()?);
        }

        Ok(MultiProof { leaves, proof, proof_flags })
    }
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let root = u.arbitrary()?;
        let single_proof = u.arbitrary()?;
        let multi_proof = u.arbitrary()?;

        Ok(Input { root, single_proof, multi_proof })
    }
}
