//! Permit Contract.
//!
//! Extension of the ERC-20 standard allowing approvals to be made
//! via signatures, as defined in the [ERC].
//!
//! Adds the `permit` method, which can be used to change an account’s ERC-20
//! allowance (see [`erc20::IErc20::allowance`]) by presenting a message signed
//! by the account. By not relying on [`erc20::IErc20::approve`], the token
//! holder account doesn’t need to send a transaction, and thus is not required
//! to hold Ether at all.
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-2612

use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, keccak256, Address, B256, U256, U8};
use alloy_sol_types::SolType;
use stylus_sdk::{block, call::MethodError, function_selector, prelude::*};

use crate::{
    token::{erc20, erc20::Erc20},
    utils::{
        cryptography::eip712::IEip712,
        nonces::{INonces, Nonces},
        precompiles::{
            primitives::ecrecover::{
                self, ECDSAInvalidSignature, ECDSAInvalidSignatureS,
            },
            Precompiles,
        },
    },
};

const PERMIT_TYPEHASH: [u8; 32] =
    keccak_const::Keccak256::new()
        .update(b"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")
        .finalize();

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    pub(crate) type StructHashTuple = sol! {
        tuple(bytes32, address, address, uint256, uint256, uint256)
    };

    sol! {
        /// Indicates an error related to the fact that
        /// permit deadline has expired.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2612ExpiredSignature(uint256 deadline);

        /// Indicates an error related to the issue about mismatched signature.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC2612InvalidSigner(address signer, address owner);
    }
}

/// A Permit error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the fact that
    /// permit deadline has expired.
    ExpiredSignature(ERC2612ExpiredSignature),
    /// Indicates an error related to the issue about mismatched signature.
    InvalidSigner(ERC2612InvalidSigner),
    /// Indicates an error related to the current balance of `sender`. Used
    /// in transfers.
    InsufficientBalance(erc20::ERC20InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(erc20::ERC20InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(erc20::ERC20InvalidReceiver),
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    InsufficientAllowance(erc20::ERC20InsufficientAllowance),
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(erc20::ERC20InvalidSpender),
    /// Indicates a failure with the `approver` of a token to be approved.
    /// Used in approvals. approver Address initiating an approval
    /// operation.
    InvalidApprover(erc20::ERC20InvalidApprover),
    /// The signature derives the [`Address::ZERO`].
    InvalidSignature(ECDSAInvalidSignature),
    /// The signature has an `S` value that is in the upper half order.
    InvalidSignatureS(ECDSAInvalidSignatureS),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<erc20::Error> for Error {
    fn from(value: erc20::Error) -> Self {
        match value {
            erc20::Error::InsufficientBalance(e) => {
                Error::InsufficientBalance(e)
            }
            erc20::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc20::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc20::Error::InsufficientAllowance(e) => {
                Error::InsufficientAllowance(e)
            }
            erc20::Error::InvalidSpender(e) => Error::InvalidSpender(e),
            erc20::Error::InvalidApprover(e) => Error::InvalidApprover(e),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<ecrecover::Error> for Error {
    fn from(value: ecrecover::Error) -> Self {
        match value {
            ecrecover::Error::InvalidSignature(e) => Error::InvalidSignature(e),
            ecrecover::Error::InvalidSignatureS(e) => {
                Error::InvalidSignatureS(e)
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc20Permit`] Contract.
#[storage]
pub struct Erc20Permit<T: IEip712 + StorageType> {
    /// Contract implementing [`IEip712`] trait.
    pub(crate) eip712: T,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl<T: IEip712 + StorageType> TopLevelStorage for Erc20Permit<T> {}

/// Interface for [`Erc20Permit`]
pub trait IErc20Permit: INonces {
    /// The error type associated to this interface.
    type Error: Into<alloc::vec::Vec<u8>>;

    // Calculated manually to include [`INonces::nonces`].
    /// Solidity interface id associated with [`IErc20Permit`] trait.
    /// Computed as a XOR of selectors for each function in the trait.
    #[must_use]
    fn interface_id() -> B32
    where
        Self: Sized,
    {
        B32::new(function_selector!("DOMAIN_SEPARATOR",))
            ^ B32::new(function_selector!("nonces", Address,))
            ^ B32::new(function_selector!(
                "permit", Address, Address, U256, U256, U8, B256, B256
            ))
    }

    /// Returns the domain separator used in the encoding of the signature for
    /// [`Self::permit`], as defined by EIP712.
    ///
    /// NOTE: The implementation should use `#[selector(name =
    /// "DOMAIN_SEPARATOR")]` to match Solidity's camelCase naming
    /// convention.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn domain_separator(&self) -> B256;

    /// Sets `value` as the allowance of `spender` over `owner`'s tokens,
    /// given `owner`'s signed approval.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state. given address.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `value` - The number of tokens being permitted to transfer by
    ///   `spender`.
    /// * `deadline` - Deadline for the permit action.
    /// * `v` - v value from the `owner`'s signature.
    /// * `r` - r value from the `owner`'s signature.
    /// * `s` - s value from the `owner`'s signature.
    ///
    /// # Errors
    ///
    /// * [`ERC2612ExpiredSignature`] - If the `deadline` param is from the
    ///   past.
    /// * [`ERC2612InvalidSigner`] - If signer is not an `owner`.
    /// * [`ecrecover::Error::InvalidSignatureS`] - If the `s` value is grater
    ///   than [`ecrecover::SIGNATURE_S_UPPER_BOUND`].
    /// * [`ecrecover::Error::InvalidSignature`] - If the recovered address is
    ///   [`Address::ZERO`].
    /// * [`erc20::Error::InvalidSpender`] - If the `spender` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`erc20::Approval`]
    #[allow(clippy::too_many_arguments)]
    fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<(), Self::Error>;
}

impl<T: IEip712 + StorageType> Erc20Permit<T> {
    /// See [`IErc20Permit::domain_separator`].
    #[must_use]
    pub fn domain_separator(&self) -> B256 {
        self.eip712.domain_separator_v4()
    }

    /// See [`IErc20Permit::permit`].
    #[allow(clippy::too_many_arguments, clippy::missing_errors_doc)]
    pub fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: B256,
        s: B256,
        erc20: &mut Erc20,
        nonces: &mut Nonces,
    ) -> Result<(), Error> {
        if U256::from(block::timestamp()) > deadline {
            return Err(ERC2612ExpiredSignature { deadline }.into());
        }

        let struct_hash = keccak256(StructHashTuple::abi_encode(&(
            PERMIT_TYPEHASH,
            owner,
            spender,
            value,
            nonces.use_nonce(owner),
            deadline,
        )));

        let hash: B256 = self.eip712.hash_typed_data_v4(struct_hash);

        let signer: Address = self.ec_recover(hash, v, r, s)?;

        if signer != owner {
            return Err(ERC2612InvalidSigner { signer, owner }.into());
        }

        erc20._approve(owner, spender, value, true)?;

        Ok(())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use alloy_primitives::{fixed_bytes, keccak256, uint, Address, B256, U256};
    use alloy_signer::SignerSync;
    use alloy_sol_types::{sol, SolType};
    use motsu::prelude::*;

    use super::*;
    use crate::{
        token::erc20::{self, Approval, Erc20, IErc20},
        utils::{
            cryptography::eip712::IEip712,
            nonces::{INonces, Nonces},
        },
    };

    #[entrypoint]
    #[storage]
    struct Erc20PermitExample {
        erc20: Erc20,
        nonces: Nonces,
        permit: Erc20Permit<Eip712>,
    }

    #[storage]
    struct Eip712;

    impl IEip712 for Eip712 {
        const NAME: &'static str = "ERC-20 Permit Example";
        const VERSION: &'static str = "1";
    }

    #[public]
    #[implements(IErc20<Error = Error>, INonces, IErc20Permit<Error = Error>)]
    impl Erc20PermitExample {
        // Add token minting feature.
        fn mint(&mut self, account: Address, value: U256) -> Result<(), Error> {
            Ok(self.erc20._mint(account, value)?)
        }
    }

    #[public]
    impl IErc20 for Erc20PermitExample {
        type Error = Error;

        fn total_supply(&self) -> U256 {
            self.erc20.total_supply()
        }

        fn balance_of(&self, account: Address) -> U256 {
            self.erc20.balance_of(account)
        }

        fn transfer(
            &mut self,
            to: Address,
            value: U256,
        ) -> Result<bool, Self::Error> {
            Ok(self.erc20.transfer(to, value)?)
        }

        fn allowance(&self, owner: Address, spender: Address) -> U256 {
            self.erc20.allowance(owner, spender)
        }

        fn approve(
            &mut self,
            spender: Address,
            value: U256,
        ) -> Result<bool, Self::Error> {
            Ok(self.erc20.approve(spender, value)?)
        }

        fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            value: U256,
        ) -> Result<bool, Self::Error> {
            Ok(self.erc20.transfer_from(from, to, value)?)
        }
    }

    #[public]
    impl INonces for Erc20PermitExample {
        fn nonces(&self, owner: Address) -> U256 {
            self.nonces.nonces(owner)
        }
    }

    #[public]
    impl IErc20Permit for Erc20PermitExample {
        type Error = Error;

        #[selector(name = "DOMAIN_SEPARATOR")]
        fn domain_separator(&self) -> B256 {
            self.permit.domain_separator()
        }

        fn permit(
            &mut self,
            owner: Address,
            spender: Address,
            value: U256,
            deadline: U256,
            v: u8,
            r: B256,
            s: B256,
        ) -> Result<(), Self::Error> {
            self.permit.permit(
                owner,
                spender,
                value,
                deadline,
                v,
                r,
                s,
                &mut self.erc20,
                &mut self.nonces,
            )
        }
    }

    // Saturday, 1 January 2000 00:00:00
    const EXPIRED_DEADLINE: U256 = uint!(946_684_800_U256);

    // Wednesday, 1 January 3000 00:00:00
    const FAIR_DEADLINE: U256 = uint!(32_503_680_000_U256);

    const PERMIT_TYPEHASH: [u8; 32] =
    keccak_const::Keccak256::new()
        .update(b"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")
        .finalize();

    type PermitStructHashTuple = sol! {
        tuple(bytes32, address, address, uint256, uint256, uint256)
    };

    fn permit_struct_hash(
        owner: impl Into<Address>,
        spender: Address,
        value: U256,
        nonce: U256,
        deadline: U256,
    ) -> B256 {
        keccak256(PermitStructHashTuple::abi_encode(&(
            PERMIT_TYPEHASH,
            owner.into(),
            spender,
            value,
            nonce,
            deadline,
        )))
    }

    fn to_typed_data_hash(domain_separator: B256, struct_hash: B256) -> B256 {
        let typed_dat_hash =
            crate::utils::cryptography::eip712::to_typed_data_hash(
                &domain_separator,
                &struct_hash,
            );

        B256::from_slice(typed_dat_hash.as_slice())
    }

    // I was unable to find a function in alloy that converts `v` into
    // [non-eip155 value], so I implemented the logic manually.
    //
    // [non-eip155 value]: https://eips.ethereum.org/EIPS/eip-155
    fn to_non_eip155_v(v: bool) -> u8 {
        u8::from(v) + 27
    }

    fn create_permit_signature(
        contract: &Contract<Erc20PermitExample>,
        signer: Account,
        owner: Address,
        spender: Address,
        value: U256,
        nonce: U256,
        deadline: U256,
    ) -> (u8, B256, B256) {
        let struct_hash =
            permit_struct_hash(owner, spender, value, nonce, deadline);

        let domain_separator = contract.sender(owner).domain_separator();

        let typed_data_hash = to_typed_data_hash(domain_separator, struct_hash);
        let signature = signer
            .signer()
            .sign_hash_sync(&B256::from_slice(typed_data_hash.as_slice()))
            .unwrap();

        (
            to_non_eip155_v(signature.v()),
            signature.r().into(),
            signature.s().into(),
        )
    }

    #[motsu::test]
    fn initial_nonce_is_zero(
        contract: Contract<Erc20PermitExample>,
        alice: Address,
    ) {
        assert_eq!(contract.sender(alice).nonces(alice), U256::ZERO);
    }

    #[motsu::test]
    fn domain_separator_is_consistent(
        contract: Contract<Erc20PermitExample>,
        alice: Address,
    ) {
        let domain_separator1 = contract.sender(alice).domain_separator();
        let domain_separator2 = contract.sender(alice).domain_separator();

        assert_eq!(domain_separator1, domain_separator2);
        assert_ne!(domain_separator1, B256::ZERO);
    }

    #[motsu::test]
    fn permit_increases_nonce_and_sets_allowance(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let value = uint!(42_U256);
        let initial_nonce = contract.sender(alice).nonces(alice.address());

        let (v, r, s) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender,
            value,
            initial_nonce,
            FAIR_DEADLINE,
        );

        // Mock the signature verification by setting up the test environment
        // to return the expected signer
        let result = contract.sender(alice).permit(
            alice.address(),
            spender,
            value,
            FAIR_DEADLINE,
            v,
            r,
            s,
        );

        assert!(result.is_ok());

        assert_eq!(
            contract.sender(alice).nonces(alice.address()),
            initial_nonce + U256::ONE
        );

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender),
            value
        );

        contract.assert_emitted(&Approval {
            owner: alice.address(),
            spender,
            value,
        });
    }

    #[motsu::test]
    fn permit_with_zero_value(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let value = U256::ZERO;
        let nonce = contract.sender(alice).nonces(alice.address());

        let (v, r, s) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender,
            value,
            nonce,
            FAIR_DEADLINE,
        );

        let result = contract.sender(alice).permit(
            alice.address(),
            spender,
            value,
            FAIR_DEADLINE,
            v,
            r,
            s,
        );

        assert!(result.is_ok());

        assert_eq!(
            contract.sender(alice).nonces(alice.address()),
            nonce + U256::ONE
        );

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender),
            U256::ZERO
        );

        contract.assert_emitted(&Approval {
            owner: alice.address(),
            spender,
            value: U256::ZERO,
        });
    }

    #[motsu::test]
    fn permit_with_maximum_value(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let value = U256::MAX;
        let nonce = contract.sender(alice).nonces(alice.address());

        let (v, r, s) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender,
            value,
            nonce,
            FAIR_DEADLINE,
        );

        let result = contract.sender(alice).permit(
            alice.address(),
            spender,
            value,
            FAIR_DEADLINE,
            v,
            r,
            s,
        );

        assert!(result.is_ok());

        assert_eq!(
            contract.sender(alice).nonces(alice.address()),
            nonce + U256::ONE
        );

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender),
            U256::MAX
        );

        contract.assert_emitted(&Approval {
            owner: alice.address(),
            spender,
            value: U256::MAX,
        });
    }

    #[motsu::test]
    fn multiple_permits_increment_nonces_correctly(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender1: Address,
        spender2: Address,
    ) {
        let value1 = uint!(100_U256);
        let value2 = uint!(200_U256);
        let initial_nonce = contract.sender(alice).nonces(alice.address());

        // First permit
        let (v1, r1, s1) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender1,
            value1,
            initial_nonce,
            FAIR_DEADLINE,
        );

        let result1 = contract.sender(alice).permit(
            alice.address(),
            spender1,
            value1,
            FAIR_DEADLINE,
            v1,
            r1,
            s1,
        );

        assert!(result1.is_ok());

        let nonce_after_first = initial_nonce + U256::ONE;
        assert_eq!(
            contract.sender(alice).nonces(alice.address()),
            nonce_after_first
        );

        // Second permit with incremented nonce
        let (v2, r2, s2) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender2,
            value2,
            nonce_after_first,
            FAIR_DEADLINE,
        );

        let result2 = contract.sender(alice).permit(
            alice.address(),
            spender2,
            value2,
            FAIR_DEADLINE,
            v2,
            r2,
            s2,
        );

        assert!(result2.is_ok());

        assert_eq!(
            contract.sender(alice).nonces(alice.address()),
            initial_nonce + uint!(2_U256)
        );

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender1),
            value1
        );

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender2),
            value2
        );
    }

    #[motsu::test]
    fn permit_overrides_existing_allowance(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let initial_value = uint!(100_U256);
        let new_value = uint!(500_U256);

        // Set initial allowance via regular approve
        let approve_result =
            contract.sender(alice).approve(spender, initial_value);
        assert!(approve_result.is_ok());

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender),
            initial_value
        );

        // Now use permit to change the allowance
        let nonce = contract.sender(alice).nonces(alice.address());
        let (v, r, s) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender,
            new_value,
            nonce,
            FAIR_DEADLINE,
        );

        let result = contract.sender(alice).permit(
            alice.address(),
            spender,
            new_value,
            FAIR_DEADLINE,
            v,
            r,
            s,
        );

        assert!(result.is_ok());

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender),
            new_value
        );

        contract.assert_emitted(&Approval {
            owner: alice.address(),
            spender,
            value: new_value,
        });
    }

    #[motsu::test]
    fn permit_rejects_expired_signature(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let value = uint!(42_U256);
        let nonce = contract.sender(alice).nonces(alice.address());

        let (v, r, s) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender,
            value,
            nonce,
            EXPIRED_DEADLINE,
        );

        let err = contract
            .sender(alice)
            .permit(alice.address(), spender, value, EXPIRED_DEADLINE, v, r, s)
            .motsu_expect_err("should return `ERC2612ExpiredSignature`");

        assert!(matches!(
            err,
            Error::ExpiredSignature(ERC2612ExpiredSignature { deadline }) if deadline == EXPIRED_DEADLINE
        ));

        assert_eq!(contract.sender(alice).nonces(alice.address()), nonce);

        assert_eq!(
            contract.sender(alice).allowance(alice.address(), spender),
            U256::ZERO
        );
    }

    #[motsu::test]
    fn permit_rejects_reused_signature(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let value = uint!(42_U256);
        let initial_nonce = contract.sender(alice).nonces(alice.address());

        let (v, r, s) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender,
            value,
            initial_nonce,
            FAIR_DEADLINE,
        );

        // First permit should succeed
        let result = contract.sender(alice).permit(
            alice.address(),
            spender,
            value,
            FAIR_DEADLINE,
            v,
            r,
            s,
        );

        assert!(result.is_ok());

        assert_eq!(
            contract.sender(alice).nonces(alice.address()),
            initial_nonce + U256::ONE
        );

        // Second permit with same signature should fail
        let err = contract
            .sender(alice)
            .permit(alice.address(), spender, value, FAIR_DEADLINE, v, r, s)
            .motsu_expect_err("should return `ERC2612InvalidSigner`");

        assert!(matches!(
            err,
            Error::InvalidSigner(ERC2612InvalidSigner { owner, .. }) if owner == alice.address()
        ));
    }

    #[motsu::test]
    fn permit_rejects_wrong_signer(
        contract: Contract<Erc20PermitExample>,
        alice: Address,
        bob: Account,
        spender: Address,
    ) {
        let value = uint!(42_U256);
        let nonce = contract.sender(alice).nonces(alice);

        // Create signature with bob's key but for alice's address
        let (v, r, s) = create_permit_signature(
            &contract,
            bob, // Wrong signer
            alice,
            spender,
            value,
            nonce,
            FAIR_DEADLINE,
        );

        let err = contract
            .sender(alice)
            .permit(alice, spender, value, FAIR_DEADLINE, v, r, s)
            .motsu_expect_err("should return `ERC2612InvalidSigner`");

        assert!(matches!(
            err,
            Error::InvalidSigner(ERC2612InvalidSigner { owner, signer }) if owner == alice && signer == bob.address()
        ));

        assert_eq!(contract.sender(alice).nonces(alice), nonce);

        assert_eq!(
            contract.sender(alice).allowance(alice, spender),
            U256::ZERO
        );
    }

    #[motsu::test]
    fn permit_rejects_zero_spender(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
    ) {
        let value = uint!(42_U256);
        let spender = Address::ZERO;
        let nonce = contract.sender(alice).nonces(alice.address());

        let (v, r, s) = create_permit_signature(
            &contract,
            alice,
            alice.address(),
            spender,
            value,
            nonce,
            FAIR_DEADLINE,
        );

        let err = contract
            .sender(alice)
            .permit(alice.address(), spender, value, FAIR_DEADLINE, v, r, s)
            .motsu_expect_err("should return `ERC20InvalidSpender`");

        assert!(matches!(
            err,
            Error::InvalidSpender(erc20::ERC20InvalidSpender { spender })
            if spender.is_zero()
        ));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc20PermitExample as IErc20Permit>::interface_id();
        let expected: B32 = fixed_bytes!("0x9d8ff7da");
        assert_eq!(actual, expected);
    }
}
