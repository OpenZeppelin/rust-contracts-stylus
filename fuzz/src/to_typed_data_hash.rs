use openzeppelin_stylus::utils::cryptography::eip712::to_typed_data_hash;

fn main() {
    afl::fuzz!(|data: ([u8; 32], [u8; 32])| {
        let (domain_separator, struct_hash) = data;
        let _ = to_typed_data_hash(&domain_separator, &struct_hash);
    })
}
