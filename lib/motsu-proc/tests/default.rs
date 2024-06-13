mod erc20 {
    use alloy_primitives::U256;
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
    fn erc20_initiates_correctly(contract: Erc20) {
        assert_eq!(contract._total_supply.get(), U256::from(0));
    }
}

mod uint {
    use alloy_primitives::{U128, U16, U256, U32, U64, U8};
    use stylus_sdk::stylus_proc::sol_storage;

    sol_storage! {
        #[derive(motsu_proc::DefaultStorageLayout)]
        pub struct UintContract {
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
    fn uint_initiates_correctly(contract: UintContract) {
        assert_eq!(contract.a.get(), U256::from(0));
        assert_eq!(contract.b.get(), U128::from(0));
        assert_eq!(contract.c.get(), U64::from(0));
        assert_eq!(contract.d.get(), U32::from(0));
        assert_eq!(contract.e.get(), U16::from(0));
        assert_eq!(contract.f.get(), U8::from(0));
        assert_eq!(contract.g.get(), U8::from(0));
    }

    #[motsu::test]
    fn uint_updates_correctly(contract: UintContract) {
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
}

mod address {
    use alloy_primitives::{address, Address};
    use stylus_sdk::stylus_proc::sol_storage;

    sol_storage! {
        #[derive(motsu_proc::DefaultStorageLayout)]
        pub struct AddressContract {
            address lender;
            address borrower;
        }
    }

    #[motsu::test]
    fn address_initiates_correctly(contract: AddressContract) {
        assert_eq!(contract.lender.get(), Address::ZERO);
        assert_eq!(contract.borrower.get(), Address::ZERO);
    }

    #[motsu::test]
    fn address_updates_correctly(contract: AddressContract) {
        let lender = address!("a935CEC3c5Ef99D7F1016674DEFd455Ef06776C5");
        let borrower = address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF");
        contract.lender.set(lender);
        contract.borrower.set(borrower);
        assert_eq!(contract.lender.get(), lender);
        assert_eq!(contract.borrower.get(), borrower);
    }
}
