use alloy_primitives::{address, Address, U128, U16, U256, U32, U64, U8};
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    #[derive(motsu_proc::DefaultStorageLayout)]
    pub struct Erc20 {
        mapping(address => uint256) _balances;
        mapping(address => mapping(address => uint256)) _allowances;
        uint256 _total_supply;
    }
}

#[motsu::test]
fn mapping_initializes_and_updates(contract: Erc20) {
    let key = address!("a935CEC3c5Ef99D7F1016674DEFd455Ef06776C5");
    let value = U256::from(100);
    contract._balances.insert(key, value);
    let balance = contract._balances.get(key);
    assert_eq!(balance, value);
}

sol_storage! {
    #[derive(motsu_proc::DefaultStorageLayout)]
    pub struct UintSorted {
        uint256 a;
        uint128 b;
        uint64 c;
        uint32 d;
        uint16 e;
        uint8 f;
        uint8 g;
    }
}

#[motsu::test]
fn uint_sorted_initializes(contract: UintSorted) {
    assert_eq!(contract.a.get(), U256::ZERO);
    assert_eq!(contract.b.get(), U128::ZERO);
    assert_eq!(contract.c.get(), U64::ZERO);
    assert_eq!(contract.d.get(), U32::ZERO);
    assert_eq!(contract.e.get(), U16::ZERO);
    assert_eq!(contract.f.get(), U8::ZERO);
    assert_eq!(contract.g.get(), U8::ZERO);
}

#[motsu::test]
fn uint_sorted_updates(contract: UintSorted) {
    contract.a.set(U256::from(1));
    contract.b.set(U128::from(2));
    contract.c.set(U64::from(3));
    contract.d.set(U32::from(4));
    contract.e.set(U16::from(5));
    contract.f.set(U8::from(6));
    contract.g.set(U8::from(7));

    assert_eq!(contract.a.get(), U256::from(1));
    assert_eq!(contract.b.get(), U128::from(2));
    assert_eq!(contract.c.get(), U64::from(3));
    assert_eq!(contract.d.get(), U32::from(4));
    assert_eq!(contract.e.get(), U16::from(5));
    assert_eq!(contract.f.get(), U8::from(6));
    assert_eq!(contract.g.get(), U8::from(7));
}

sol_storage! {
    #[derive(motsu_proc::DefaultStorageLayout)]
    pub struct UintUnsorted {
        uint256 a;
        uint16 b;
        uint64 c;
        uint8 d;
        uint32 e;
        uint128 f;
        uint8 g;
    }
}

#[motsu::test]
fn uint_unsorted_initializes(contract: UintUnsorted) {
    assert_eq!(contract.a.get(), U256::ZERO);
    assert_eq!(contract.b.get(), U16::ZERO);
    assert_eq!(contract.c.get(), U64::ZERO);
    assert_eq!(contract.d.get(), U8::ZERO);
    assert_eq!(contract.e.get(), U32::ZERO);
    assert_eq!(contract.f.get(), U128::ZERO);
    assert_eq!(contract.g.get(), U8::ZERO);
}

#[motsu::test]
fn uint_unsorted_updates(contract: UintUnsorted) {
    contract.a.set(U256::from(1));
    contract.b.set(U16::from(5));
    contract.c.set(U64::from(3));
    contract.d.set(U8::from(6));
    contract.e.set(U32::from(4));
    contract.f.set(U128::from(2));
    contract.g.set(U8::from(7));

    assert_eq!(contract.a.get(), U256::from(1));
    assert_eq!(contract.b.get(), U16::from(5));
    assert_eq!(contract.c.get(), U64::from(3));
    assert_eq!(contract.d.get(), U8::from(6));
    assert_eq!(contract.e.get(), U32::from(4));
    assert_eq!(contract.f.get(), U128::from(2));
    assert_eq!(contract.g.get(), U8::from(7));
}

sol_storage! {
    #[derive(motsu_proc::DefaultStorageLayout)]
    pub struct AddressContract {
        address lender;
        address borrower;
    }
}

#[motsu::test]
fn address_initializes(contract: AddressContract) {
    assert_eq!(contract.lender.get(), Address::ZERO);
    assert_eq!(contract.borrower.get(), Address::ZERO);
}

#[motsu::test]
fn address_updates(contract: AddressContract) {
    let lender = address!("a935CEC3c5Ef99D7F1016674DEFd455Ef06776C5");
    let borrower = address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF");
    contract.lender.set(lender);
    contract.borrower.set(borrower);
    assert_eq!(contract.lender.get(), lender);
    assert_eq!(contract.borrower.get(), borrower);
}
