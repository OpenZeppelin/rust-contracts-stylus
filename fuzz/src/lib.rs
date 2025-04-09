use rs_merkle::{algorithms::Keccak256, Hasher};

#[derive(Clone)]
pub struct CommutativeKeccak256;

impl Hasher for CommutativeKeccak256 {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> Self::Hash {
        Keccak256::hash(data)
    }

    fn concat_and_hash(
        left: &Self::Hash,
        right: Option<&Self::Hash>,
    ) -> Self::Hash {
        match right {
            Some(right) => {
                if left > right {
                    let concat = &[right.as_slice(), left.as_slice()].concat();
                    Self::hash(concat)
                } else {
                    let concat = &[left.as_slice(), right.as_slice()].concat();
                    Self::hash(concat)
                }
            }
            None => *left,
        }
    }
}

pub mod consts {
    pub mod merkle {
        pub const MIN_LEAVES: usize = 2;
        pub const MAX_LEAVES: usize = 32;
        pub const MIN_INDICES: usize = 1;
    }
}
