#![no_main]

use openzeppelin_stylus::utils::cryptography::eip712::to_typed_data_hash;

#[motsu::fuzz]
fn test(domain_separator: [u8; 32], struct_hash: [u8; 32]) {
    let _ = to_typed_data_hash(&domain_separator, &struct_hash);
}
