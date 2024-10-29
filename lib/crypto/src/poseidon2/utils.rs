// NOTE#q: this utils ported from root of poseidon2 crate

use crypto_bigint::Random;

use crate::field::prime::PrimeField;

pub fn random_scalar<F: PrimeField + Random>() -> F {
    let mut rng = rand::thread_rng();
    F::random(&mut rng)
}
