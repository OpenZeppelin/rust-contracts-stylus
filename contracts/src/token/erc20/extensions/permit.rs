//! Permit Contract.
//!
//! Extension of the ERC-20 standard allowing approvals to be made
//! via signatures, as defined in the [ERC].
//!
//! Adds the `permit` method, which can be used to change an account’s
//! ERC20 allowance (see [`crate::token::erc20::IErc20::allowance`])
//! by presenting a message signed by the account.
//! By not relying on [`erc20::IErc20::approve`],
//! the token holder account doesn’t need to send a transaction,
//! and thus is not required to hold Ether at all.
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-2612

use alloc::{vec, vec::Vec};

use alloy_primitives::{keccak256, Address, FixedBytes, B256, U256, U8};
use alloy_sol_types::SolType;
use stylus_sdk::{block, call::MethodError, function_selector, prelude::*};

use crate::{
    token::erc20::{self, Erc20},
    utils::{
        cryptography::{
            ecdsa::{self, ECDSAInvalidSignature, ECDSAInvalidSignatureS},
            eip712::IEip712,
        },
        nonces::{INonces, Nonces},
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
    /// Indicates an error related to the current balance of `sender`. Used in
    /// transfers.
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
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals. approver Address initiating an approval operation.
    InvalidApprover(erc20::ERC20InvalidApprover),
    /// The signature derives the [`Address::ZERO`].
    InvalidSignature(ECDSAInvalidSignature),
    /// The signature has an `S` value that is in the upper half order.
    InvalidSignatureS(ECDSAInvalidSignatureS),
}

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

impl From<ecdsa::Error> for Error {
    fn from(value: ecdsa::Error) -> Self {
        match value {
            ecdsa::Error::InvalidSignature(e) => Error::InvalidSignature(e),
            ecdsa::Error::InvalidSignatureS(e) => Error::InvalidSignatureS(e),
        }
    }
}

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
    fn interface_id() -> FixedBytes<4>
    where
        Self: Sized,
    {
        FixedBytes::<4>::new(function_selector!("DOMAIN_SEPARATOR",))
            ^ FixedBytes::<4>::new(function_selector!("nonces", Address,))
            ^ FixedBytes::<4>::new(function_selector!(
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
    /// * [`ecdsa::Error::InvalidSignatureS`] - If the `s` value is grater than
    ///   [`ecdsa::SIGNATURE_S_UPPER_BOUND`].
    /// * [`ecdsa::Error::InvalidSignature`] - If the recovered address is
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

        let signer: Address = ecdsa::recover(self, hash, v, r, s)?;

        if signer != owner {
            return Err(ERC2612InvalidSigner { signer, owner }.into());
        }

        erc20._approve(owner, spender, value, true)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_signer::{Result, Signature, SignerSync};
    use alloy_sol_macro::sol;
    use motsu::prelude::*;

    use super::*;
    use crate::token::erc20::IErc20;

    #[storage]
    struct Eip712;

    impl IEip712 for Eip712 {
        const NAME: &'static str = "ERC-20 Permit Example";
        const VERSION: &'static str = "1";
    }

    #[storage]
    struct Erc20PermitExample {
        erc20: Erc20,
        nonces: Nonces,
        erc20_permit: Erc20Permit<Eip712>,
    }

    unsafe impl TopLevelStorage for Erc20PermitExample {}

    #[public]
    #[implements(INonces, IErc20Permit<Error = Error>)]
    impl Erc20PermitExample {
        fn mint(&mut self, account: Address, value: U256) -> Result<(), Error> {
            Ok(self.erc20._mint(account, value)?)
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
            self.erc20_permit.domain_separator()
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
            self.erc20_permit.permit(
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

    const PERMIT_TYPEHASH: [u8; 32] =
    keccak_const::Keccak256::new()
        .update(b"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")
        .finalize();

    type PermitStructHashTuple = sol! {
        tuple(bytes32, address, address, uint256, uint256, uint256)
    };

    fn to_typed_data_hash(domain_separator: B256, struct_hash: B256) -> B256 {
        let typed_dat_hash =
            crate::utils::cryptography::eip712::to_typed_data_hash(
                &domain_separator,
                &struct_hash,
            );

        B256::from_slice(typed_dat_hash.as_slice())
    }

    fn permit_struct_hash(
        owner: Address,
        spender: Address,
        value: U256,
        nonce: U256,
        deadline: U256,
    ) -> B256 {
        keccak256(PermitStructHashTuple::abi_encode(&(
            PERMIT_TYPEHASH,
            owner,
            spender,
            value,
            nonce,
            deadline,
        )))
    }

    fn sign_permit_hash(
        permit_hash: &B256,
        signer_account: Account,
    ) -> Result<Signature> {
        signer_account.signer().sign_hash_sync(permit_hash)
    }

    // I was unable to find a function in alloy that converts `v` into
    // [non-eip155 value], so I implemented the logic manually.
    //
    // [non-eip155 value]: https://eips.ethereum.org/EIPS/eip-155
    fn to_non_eip155_v(v: bool) -> u8 {
        v as u8 + 27
    }

    #[motsu::test]
    fn error_when_expired_deadline_for_permit(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() - 3600); // 1 hour ago

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        let err = contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect_err("should return `ERC2612ExpiredSignature`");

        assert!(matches!(
            err,
            Error::ExpiredSignature(ERC2612ExpiredSignature {
                deadline
            }) if deadline == deadline
        ));
    }

    #[motsu::test]
    fn error_when_invalid_signer_for_permit(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        bob: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600); // 1 hour from now

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        // Sign with bob instead of alice
        let signature =
            sign_permit_hash(&typed_data_hash, bob).expect("should sign");

        let err = contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect_err("should return `ERC2612InvalidSigner`");

        assert!(matches!(
            err,
            Error::InvalidSigner(ERC2612InvalidSigner {
                signer,
                owner
            }) if signer == bob.address() && owner == alice.address()
        ));
    }

    #[motsu::test]
    fn error_when_invalid_signature_for_permit(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        // Corrupt the signature by modifying r
        let mut corrupted_r = signature.r();
        corrupted_r = corrupted_r.wrapping_add(U256::from(1));

        let err = contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                corrupted_r.into(),
                signature.s().into(),
            )
            .expect_err("should return `InvalidSignature`");

        assert!(matches!(err, Error::InvalidSignature(_)));
    }

    #[motsu::test]
    fn error_when_invalid_signature_s_value_for_permit(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        // Create an invalid S value (upper half order)
        let invalid_s = B256::from_slice(&[0xff; 32]);

        let err = contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                invalid_s,
            )
            .expect_err("should return `InvalidSignatureS`");

        assert!(matches!(err, Error::InvalidSignatureS(_)));
    }

    #[motsu::test]
    fn success_when_valid_permit_with_zero_value(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let permit_value = U256::ZERO;
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            permit_value,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                permit_value,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect("should permit");

        assert_eq!(
            contract.sender(alice).erc20.allowance(alice.address(), spender),
            permit_value
        );
    }

    #[motsu::test]
    fn success_when_valid_permit_with_max_value(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::MAX;
        let permit_value = U256::MAX;
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            permit_value,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                permit_value,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect("should permit");

        assert_eq!(
            contract.sender(alice).erc20.allowance(alice.address(), spender),
            permit_value
        );
    }

    #[motsu::test]
    fn error_when_reusing_nonce_for_permit(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        // First permit should succeed
        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect("should permit");

        // Second permit with same signature should fail
        let err = contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect_err("should return `InvalidSignature`");

        assert!(matches!(err, Error::InvalidSignature(_)));
    }

    #[motsu::test]
    fn success_when_permit_with_different_spenders(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender1: Address,
        spender2: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        // Permit for spender1
        let struct_hash1 = permit_struct_hash(
            alice.address(),
            spender1,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash1 = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash1,
        );
        let signature1 =
            sign_permit_hash(&typed_data_hash1, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender1,
                balance,
                deadline,
                to_non_eip155_v(signature1.v()),
                signature1.r().into(),
                signature1.s().into(),
            )
            .expect("should permit");

        // Permit for spender2
        let struct_hash2 = permit_struct_hash(
            alice.address(),
            spender2,
            balance,
            U256::from(1), // nonce should be 1 now
            deadline,
        );

        let typed_data_hash2 = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash2,
        );
        let signature2 =
            sign_permit_hash(&typed_data_hash2, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender2,
                balance,
                deadline,
                to_non_eip155_v(signature2.v()),
                signature2.r().into(),
                signature2.s().into(),
            )
            .expect("should permit");
    }

    #[motsu::test]
    fn success_when_permit_with_different_values(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(100);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        // First permit for 50 tokens
        let value1 = U256::from(50);
        let struct_hash1 = permit_struct_hash(
            alice.address(),
            spender,
            value1,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash1 = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash1,
        );
        let signature1 =
            sign_permit_hash(&typed_data_hash1, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                value1,
                deadline,
                to_non_eip155_v(signature1.v()),
                signature1.r().into(),
                signature1.s().into(),
            )
            .expect("should permit");

        // Second permit for remaining 50 tokens
        let value2 = U256::from(50);
        let struct_hash2 = permit_struct_hash(
            alice.address(),
            spender,
            value2,
            U256::from(1), // nonce should be 1 now
            deadline,
        );

        let typed_data_hash2 = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash2,
        );
        let signature2 =
            sign_permit_hash(&typed_data_hash2, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                value2,
                deadline,
                to_non_eip155_v(signature2.v()),
                signature2.r().into(),
                signature2.s().into(),
            )
            .expect("should permit");
    }

    #[motsu::test]
    fn error_when_permit_with_wrong_nonce(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        // Create permit with wrong nonce (should be 0, but using 1)
        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::from(1), // wrong nonce
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        let err = contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect_err("should return `InvalidSignature`");

        assert!(matches!(err, Error::InvalidSignature(_)));
    }

    #[motsu::test]
    fn success_when_permit_with_zero_value(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        // Permit for zero value
        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            U256::ZERO,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                U256::ZERO,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect("should permit");
    }

    #[motsu::test]
    fn error_when_permit_with_modified_signature(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::from(block::timestamp() + 3600);

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        // Modify the signature by changing the 's' value
        let modified_s = B256::from_slice(&[0xff; 32]);

        let err = contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                modified_s,
            )
            .expect_err("should return `InvalidSignature`");

        assert!(matches!(err, Error::InvalidSignature(_)));
    }

    #[motsu::test]
    fn success_when_permit_with_max_deadline(
        contract: Contract<Erc20PermitExample>,
        alice: Account,
        spender: Address,
    ) {
        let balance = U256::from(10);
        let deadline = U256::MAX; // Maximum deadline

        contract
            .sender(alice)
            .erc20
            ._mint(alice.address(), balance)
            .expect("should mint");

        let struct_hash = permit_struct_hash(
            alice.address(),
            spender,
            balance,
            U256::ZERO,
            deadline,
        );

        let typed_data_hash = to_typed_data_hash(
            contract.sender(alice).domain_separator(),
            struct_hash,
        );
        let signature =
            sign_permit_hash(&typed_data_hash, alice).expect("should sign");

        contract
            .sender(alice)
            .permit(
                alice.address(),
                spender,
                balance,
                deadline,
                to_non_eip155_v(signature.v()),
                signature.r().into(),
                signature.s().into(),
            )
            .expect("should permit");
    }
}
