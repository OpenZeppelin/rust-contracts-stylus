use crate::{
    field::instance::FpGoldiLocks, fp_from_hex,
    poseidon2::params::PoseidonParams,
};

type Scalar = FpGoldiLocks;

pub struct Goldilocks12Params;

#[rustfmt::skip]
impl PoseidonParams<Scalar> for Goldilocks12Params {
    const T: usize = 12;
    const D: u8 = 7;
    const ROUNDS_F: usize = 8;
    const ROUNDS_P: usize = 22;
    const MAT_INTERNAL_DIAG_M_1: &'static [Scalar] = &[
        fp_from_hex!("c3b6c08e23ba9300"),
        fp_from_hex!("d84b5de94a324fb6"),
        fp_from_hex!("0d0c371c5b35b84f"),
        fp_from_hex!("7964f570e7188037"),
        fp_from_hex!("5daf18bbd996604b"),
        fp_from_hex!("6743bc47b9595257"),
        fp_from_hex!("5528b9362c59bb70"),
        fp_from_hex!("ac45e25b7127b68b"),
        fp_from_hex!("a2077d7dfbb606b5"),
        fp_from_hex!("f3faac6faee378ae"),
        fp_from_hex!("0c6388b51545e883"),
        fp_from_hex!("d27dbb6944917b60"),
    ];
    const ROUND_CONSTANTS: &'static [&'static [Scalar]] = &[
        &[
            fp_from_hex!("13dcf33aba214f46"),
            fp_from_hex!("30b3b654a1da6d83"),
            fp_from_hex!("1fc634ada6159b56"),
            fp_from_hex!("937459964dc03466"),
            fp_from_hex!("edd2ef2ca7949924"),
            fp_from_hex!("ede9affde0e22f68"),
            fp_from_hex!("8515b9d6bac9282d"),
            fp_from_hex!("6b5c07b4e9e900d8"),
            fp_from_hex!("1ec66368838c8a08"),
            fp_from_hex!("9042367d80d1fbab"),
            fp_from_hex!("400283564a3c3799"),
            fp_from_hex!("4a00be0466bca75e"),
        ],
        &[
            fp_from_hex!("7913beee58e3817f"),
            fp_from_hex!("f545e88532237d90"),
            fp_from_hex!("22f8cb8736042005"),
            fp_from_hex!("6f04990e247a2623"),
            fp_from_hex!("fe22e87ba37c38cd"),
            fp_from_hex!("d20e32c85ffe2815"),
            fp_from_hex!("117227674048fe73"),
            fp_from_hex!("4e9fb7ea98a6b145"),
            fp_from_hex!("e0866c232b8af08b"),
            fp_from_hex!("00bbc77916884964"),
            fp_from_hex!("7031c0fb990d7116"),
            fp_from_hex!("240a9e87cf35108f"),
        ],
        &[
            fp_from_hex!("2e6363a5a12244b3"),
            fp_from_hex!("5e1c3787d1b5011c"),
            fp_from_hex!("4132660e2a196e8b"),
            fp_from_hex!("3a013b648d3d4327"),
            fp_from_hex!("f79839f49888ea43"),
            fp_from_hex!("fe85658ebafe1439"),
            fp_from_hex!("b6889825a14240bd"),
            fp_from_hex!("578453605541382b"),
            fp_from_hex!("4508cda8f6b63ce9"),
            fp_from_hex!("9c3ef35848684c91"),
            fp_from_hex!("0812bde23c87178c"),
            fp_from_hex!("fe49638f7f722c14"),
        ],
        &[
            fp_from_hex!("8e3f688ce885cbf5"),
            fp_from_hex!("b8e110acf746a87d"),
            fp_from_hex!("b4b2e8973a6dabef"),
            fp_from_hex!("9e714c5da3d462ec"),
            fp_from_hex!("6438f9033d3d0c15"),
            fp_from_hex!("24312f7cf1a27199"),
            fp_from_hex!("23f843bb47acbf71"),
            fp_from_hex!("9183f11a34be9f01"),
            fp_from_hex!("839062fbb9d45dbf"),
            fp_from_hex!("24b56e7e6c2e43fa"),
            fp_from_hex!("e1683da61c962a72"),
            fp_from_hex!("a95c63971a19bfa7"),
        ],
        &[
            fp_from_hex!("4adf842aa75d4316"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("f8fbb871aa4ab4eb"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("68e85b6eb2dd6aeb"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("07a0b06b2d270380"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("d94e0228bd282de4"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("8bdd91d3250c5278"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("209c68b88bba778f"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("b5e18cdab77f3877"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("b296a3e808da93fa"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("8370ecbda11a327e"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("3f9075283775dad8"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("b78095bb23c6aa84"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("3f36b9fe72ad4e5f"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("69bc96780b10b553"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("3f1d341f2eb7b881"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("4e939e9815838818"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("da366b3ae2a31604"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("bc89db1e7287d509"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("6102f411f9ef5659"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("58725c5e7ac1f0ab"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("0df5856c798883e7"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("f7bb62a8da4c961b"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
            fp_from_hex!("0000000000000000"),
        ],
        &[
            fp_from_hex!("c68be7c94882a24d"),
            fp_from_hex!("af996d5d5cdaedd9"),
            fp_from_hex!("9717f025e7daf6a5"),
            fp_from_hex!("6436679e6e7216f4"),
            fp_from_hex!("8a223d99047af267"),
            fp_from_hex!("bb512e35a133ba9a"),
            fp_from_hex!("fbbf44097671aa03"),
            fp_from_hex!("f04058ebf6811e61"),
            fp_from_hex!("5cca84703fac7ffb"),
            fp_from_hex!("9b55c7945de6469f"),
            fp_from_hex!("8e05bf09808e934f"),
            fp_from_hex!("2ea900de876307d7"),
        ],
        &[
            fp_from_hex!("7748fff2b38dfb89"),
            fp_from_hex!("6b99a676dd3b5d81"),
            fp_from_hex!("ac4bb7c627cf7c13"),
            fp_from_hex!("adb6ebe5e9e2f5ba"),
            fp_from_hex!("2d33378cafa24ae3"),
            fp_from_hex!("1e5b73807543f8c2"),
            fp_from_hex!("09208814bfebb10f"),
            fp_from_hex!("782e64b6bb5b93dd"),
            fp_from_hex!("add5a48eac90b50f"),
            fp_from_hex!("add4c54c736ea4b1"),
            fp_from_hex!("d58dbb86ed817fd8"),
            fp_from_hex!("6d5ed1a533f34ddd"),
        ],
        &[
            fp_from_hex!("28686aa3e36b7cb9"),
            fp_from_hex!("591abd3476689f36"),
            fp_from_hex!("047d766678f13875"),
            fp_from_hex!("a2a11112625f5b49"),
            fp_from_hex!("21fd10a3f8304958"),
            fp_from_hex!("f9b40711443b0280"),
            fp_from_hex!("d2697eb8b2bde88e"),
            fp_from_hex!("3493790b51731b3f"),
            fp_from_hex!("11caf9dd73764023"),
            fp_from_hex!("7acfb8f72878164e"),
            fp_from_hex!("744ec4db23cefc26"),
            fp_from_hex!("1e00e58f422c6340"),
        ],
        &[
            fp_from_hex!("21dd28d906a62dda"),
            fp_from_hex!("f32a46ab5f465b5f"),
            fp_from_hex!("bfce13201f3f7e6b"),
            fp_from_hex!("f30d2e7adb5304e2"),
            fp_from_hex!("ecdf4ee4abad48e9"),
            fp_from_hex!("f94e82182d395019"),
            fp_from_hex!("4ee52e3744d887c5"),
            fp_from_hex!("a1341c7cac0083b2"),
            fp_from_hex!("2302fb26c30c834a"),
            fp_from_hex!("aea3c587273bf7d3"),
            fp_from_hex!("f798e24961823ec7"),
            fp_from_hex!("962deba3e9a2cd94"),
        ],
    ];
}

#[allow(unused_imports)]
#[cfg(test)]
mod poseidon2_tests_goldilocks {
    use crate::{
        field::instance::FpGoldiLocks,
        fp_from_hex,
        poseidon2::{instance::goldilocks::Goldilocks12Params, *},
    };

    type Scalar = FpGoldiLocks;

    static TESTRUNS: usize = 5;

    #[test]
    fn consistent_perm() {
        let instance = Box::new(Poseidon2::<Goldilocks12Params, _>::new());
        let t = Goldilocks12Params::T;
        for _ in 0..TESTRUNS {
            let input1: Vec<Scalar> = (0..t).map(|_| random_scalar()).collect();

            let mut input2: Vec<Scalar>;
            loop {
                input2 = (0..t).map(|_| random_scalar()).collect();
                if input1 != input2 {
                    break;
                }
            }

            let perm1 = instance.permutation(&input1);
            let perm2 = instance.permutation(&input1);
            let perm3 = instance.permutation(&input2);
            assert_eq!(perm1, perm2);
            assert_ne!(perm1, perm3);
        }
    }

    #[test]
    fn kats() {
        let poseidon2 = Poseidon2::<Goldilocks12Params, _>::new();
        let mut input: Vec<Scalar> = vec![];
        for i in 0..Goldilocks12Params::T {
            input.push(Scalar::from(i as u64));
        }
        let perm = poseidon2.permutation(&input);
        assert_eq!(perm[0], fp_from_hex!("01eaef96bdf1c0c1"));
        assert_eq!(perm[1], fp_from_hex!("1f0d2cc525b2540c"));
        assert_eq!(perm[2], fp_from_hex!("6282c1dfe1e0358d"));
        assert_eq!(perm[3], fp_from_hex!("e780d721f698e1e6"));
        assert_eq!(perm[4], fp_from_hex!("280c0b6f753d833b"));
        assert_eq!(perm[5], fp_from_hex!("1b942dd5023156ab"));
        assert_eq!(perm[6], fp_from_hex!("43f0df3fcccb8398"));
        assert_eq!(perm[7], fp_from_hex!("e8e8190585489025"));
        assert_eq!(perm[8], fp_from_hex!("56bdbf72f77ada22"));
        assert_eq!(perm[9], fp_from_hex!("7911c32bf9dcd705"));
        assert_eq!(perm[10], fp_from_hex!("ec467926508fbe67"));
        assert_eq!(perm[11], fp_from_hex!("6a50450ddf85a6ed"));
    }
}
