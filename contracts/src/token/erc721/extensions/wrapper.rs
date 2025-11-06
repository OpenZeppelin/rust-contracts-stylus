//! Extension of the ERC-721 token contract to support token wrapping.
//!
//! Users can deposit and withdraw an "underlying token" and receive a "wrapped
//! token" with a matching token ID. This is useful in conjunction with other
//! modules.
use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use alloy_sol_types::SolError;
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    abi::Bytes,
    call::{self, Call, MethodError},
    contract, msg,
    prelude::*,
    storage::StorageAddress,
};

use crate::token::erc721::{
    self, abi::Erc721Interface, receiver::IErc721Receiver, Erc721,
    RECEIVER_FN_SELECTOR,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The received ERC-721 token couldn't be wrapped.
        ///
        /// * `token` - The token address.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721UnsupportedToken(address token);

        /// An operation with an ERC-721 token failed.
        ///
        /// * `token` - Address of the ERC-721 token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error Erc721FailedOperation(address token);

    }
}

/// An [`Erc721Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates that an address can't be an owner.
    /// For example, [`Address::ZERO`] is a forbidden owner in [`Erc721`].
    /// Used in balance queries.
    InvalidOwner(erc721::ERC721InvalidOwner),
    /// Indicates a `token_id` whose `owner` is the zero address.
    NonexistentToken(erc721::ERC721NonexistentToken),
    /// Indicates an error related to the ownership over a particular token.
    /// Used in transfers.
    IncorrectOwner(erc721::ERC721IncorrectOwner),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(erc721::ERC721InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(erc721::ERC721InvalidReceiver),
    /// Indicates a failure with the token `receiver`, with the reason
    /// specified by it.
    InvalidReceiverWithReason(erc721::InvalidReceiverWithReason),
    /// Indicates a failure with the `operator`’s approval. Used in transfers.
    InsufficientApproval(erc721::ERC721InsufficientApproval),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    InvalidApprover(erc721::ERC721InvalidApprover),
    /// Indicates a failure with the `operator` to be approved. Used in
    /// approvals.
    InvalidOperator(erc721::ERC721InvalidOperator),
    /// The received ERC-721 token couldn't be wrapped.
    UnsupportedToken(ERC721UnsupportedToken),
    /// An operation with an ERC-721 token failed.
    Erc721FailedOperation(Erc721FailedOperation),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<erc721::Error> for Error {
    fn from(value: erc721::Error) -> Self {
        match value {
            erc721::Error::InvalidOwner(e) => Error::InvalidOwner(e),
            erc721::Error::NonexistentToken(e) => Error::NonexistentToken(e),
            erc721::Error::IncorrectOwner(e) => Error::IncorrectOwner(e),
            erc721::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc721::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc721::Error::InvalidReceiverWithReason(e) => {
                Error::InvalidReceiverWithReason(e)
            }
            erc721::Error::InsufficientApproval(e) => {
                Error::InsufficientApproval(e)
            }
            erc721::Error::InvalidApprover(e) => Error::InvalidApprover(e),
            erc721::Error::InvalidOperator(e) => Error::InvalidOperator(e),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc721Wrapper`] token.
#[storage]
pub struct Erc721Wrapper {
    /// Address of the underlying token.
    underlying: StorageAddress,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc721Wrapper {}

/// Interface of an extension of the ERC-721 token contract that supports token
/// wrapping.
#[interface_id]
pub trait IErc721Wrapper: IErc721Receiver {
    /// Allow a user to deposit underlying tokens and mint the corresponding
    /// `token_ids`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to deposit tokens to.
    /// * `token_ids` - List of underlying token ids to deposit.
    ///
    /// # Errors
    ///
    /// Returns the raw revert data from the underlying ERC-721 token if the
    /// operation fails, propagating underlying token errors directly to align
    /// with Solidity behavior.
    fn deposit_for(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
    ) -> Result<bool, Vec<u8>>;

    /// Allow a user to burn wrapped tokens and withdraw the corresponding
    /// `token_ids` of the underlying tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to withdraw tokens to.
    /// * `token_ids` - List of underlying token ids to withdraw.
    ///
    /// # Errors
    ///
    /// Returns the raw revert data from the underlying ERC-721 token if the
    /// operation fails, propagating underlying token errors directly to align
    /// with Solidity behavior.
    fn withdraw_to(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
    ) -> Result<bool, Vec<u8>>;

    /// Returns the underlying token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn underlying(&self) -> Address;
}

impl Erc721Wrapper {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `underlying_token` - The wrapped token.
    pub fn constructor(&mut self, underlying_token: Address) {
        self.underlying.set(underlying_token);
    }

    /// Check [`IErc721Wrapper::deposit_for()`] for more information.
    #[allow(clippy::missing_errors_doc)]
    pub fn deposit_for(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Vec<u8>> {
        let sender = msg::sender();
        let contract_address = contract::address();
        let underlying = Erc721Interface::new(self.underlying());

        for token_id in token_ids {
            // This is an "unsafe" transfer that doesn't call any hook on
            // the receiver. With [`IErc721Wrapper::underlying()`] being trusted
            // (by design of this contract) and no other contracts expected to
            // be called from there, we are safe.
            underlying
                .transfer_from(
                    Call::new_in(self),
                    sender,
                    contract_address,
                    token_id,
                )
                .map_err(|e| match e {
                    call::Error::Revert(reason) => {
                        // Propagate underlying token errors directly.
                        reason
                    }
                    call::Error::AbiDecodingFailed(_) => {
                        // For non-revert errors, return empty bytes to
                        // indicate failure.
                        vec![]
                    }
                })?;

            erc721._safe_mint(account, token_id, &vec![].into())?;
        }

        Ok(true)
    }

    /// Check [`IErc721Wrapper::withdraw_to()`] for more information.
    #[allow(clippy::missing_errors_doc)]
    pub fn withdraw_to(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        erc721: &mut Erc721,
    ) -> Result<bool, Vec<u8>> {
        let sender = msg::sender();
        let underlying = Erc721Interface::new(self.underlying());

        for token_id in token_ids {
            // Setting the `auth` argument enables the `_is_authorized` check
            // which verifies that the token exists (from != 0).
            // Therefore, it is not needed to verify that the return value is
            // not 0 here.
            erc721._update(Address::ZERO, token_id, sender)?;

            underlying
                .safe_transfer_from(
                    Call::new_in(self),
                    contract::address(),
                    account,
                    token_id,
                    vec![].into(),
                )
                .map_err(|e| match e {
                    call::Error::Revert(reason) => {
                        // Propagate underlying token errors directly.
                        reason
                    }
                    call::Error::AbiDecodingFailed(_) => {
                        // For non-revert errors, return empty bytes to
                        // indicate failure.
                        vec![]
                    }
                })?;
        }

        Ok(true)
    }

    /// Allow minting on direct ERC-721 transfers to this contract.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - The operator of the transfer.
    /// * `from` - The address that previously owned the token.
    /// * `token_id` - The id of the token to mint.
    /// * `data` - Additional data with no specified format.
    /// * `erc721` - Write access to an [`Erc721`] contract.
    ///
    /// # Errors
    ///
    /// Returns the raw revert data if the operation fails.
    pub fn on_erc721_received(
        &mut self,
        _operator: Address,
        from: Address,
        token_id: U256,
        _data: &Bytes,
        erc721: &mut Erc721,
    ) -> Result<B32, Vec<u8>> {
        let sender = msg::sender();
        if self.underlying() != sender {
            return Err(ERC721UnsupportedToken { token: sender }.abi_encode());
        }

        erc721._safe_mint(from, token_id, &vec![].into())?;

        Ok(RECEIVER_FN_SELECTOR)
    }

    /// Check [`IErc721Wrapper::underlying()`] for more information.
    #[must_use]
    pub fn underlying(&self) -> Address {
        self.underlying.get()
    }

    /// Mints wrapped tokens to cover any underlying tokens that would have
    /// been transferred to this contract outside the deposit mechanism.
    /// This is a recovery function that can be exposed with access control
    /// if desired.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to mint tokens to.
    /// * `token_id` - A mutable reference to the Erc20 contract.
    /// * `erc721` - Write access to an [`Erc721`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::Erc721FailedOperation`] - If the underlying token is not a
    ///   [`Erc721`] contract, or the contract fails to execute the call.
    /// * [`Error::IncorrectOwner`] - If the underlying token is not owned by
    ///   the contract.
    /// * [`Error::InvalidSender`] - If `token_id` already exists.
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`Error::InvalidReceiver`] - If
    ///   [`erc721::IErc721Receiver::on_erc721_received`] hasn't returned its
    ///   interface id or returned with an error.
    fn _recover(
        &mut self,
        account: Address,
        token_id: U256,
        erc721: &mut Erc721,
    ) -> Result<U256, Error> {
        let underlying = Erc721Interface::new(self.underlying());

        let owner = underlying.owner_of(Call::new_in(self), token_id).map_err(
            |_| {
                Error::Erc721FailedOperation(Erc721FailedOperation {
                    token: self.underlying(),
                })
            },
        )?;

        let contract_address = contract::address();
        if owner != contract_address {
            return Err(erc721::Error::IncorrectOwner(
                erc721::ERC721IncorrectOwner {
                    sender: contract_address,
                    token_id,
                    owner,
                },
            )
            .into());
        }

        erc721._safe_mint(account, token_id, &vec![].into())?;

        Ok(token_id)
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::SolError;
    use motsu::prelude::*;
    use stylus_sdk::abi::Bytes;

    use super::*;
    use crate::{
        token::erc721::{
            self, receiver::tests::EmptyReasonReceiver721, IErc721,
        },
        utils::introspection::erc165::IErc165,
    };

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(U256::from).collect()
    }

    #[storage]
    struct Erc721WrapperTestExample {
        wrapper: Erc721Wrapper,
        erc721: Erc721,
    }

    unsafe impl TopLevelStorage for Erc721WrapperTestExample {}

    #[public]
    #[implements(IErc721<Error = erc721::Error>, IErc721Wrapper, IErc165)]
    impl Erc721WrapperTestExample {
        #[constructor]
        fn constructor(&mut self, underlying_token: Address) {
            self.wrapper.constructor(underlying_token);
        }

        fn recover(
            &mut self,
            account: Address,
            token_id: U256,
        ) -> Result<U256, Error> {
            self.wrapper._recover(account, token_id, &mut self.erc721)
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc721 for Erc721WrapperTestExample {
        type Error = erc721::Error;

        fn balance_of(&self, owner: Address) -> Result<U256, erc721::Error> {
            self.erc721.balance_of(owner)
        }

        fn owner_of(&self, token_id: U256) -> Result<Address, erc721::Error> {
            self.erc721.owner_of(token_id)
        }

        fn safe_transfer_from(
            &mut self,
            from: Address,
            to: Address,
            token_id: U256,
        ) -> Result<(), erc721::Error> {
            self.erc721.safe_transfer_from(from, to, token_id)
        }

        fn safe_transfer_from_with_data(
            &mut self,
            from: Address,
            to: Address,
            token_id: U256,
            data: Bytes,
        ) -> Result<(), erc721::Error> {
            self.erc721.safe_transfer_from_with_data(from, to, token_id, data)
        }

        fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            token_id: U256,
        ) -> Result<(), erc721::Error> {
            self.erc721.transfer_from(from, to, token_id)
        }

        fn approve(
            &mut self,
            to: Address,
            token_id: U256,
        ) -> Result<(), erc721::Error> {
            self.erc721.approve(to, token_id)
        }

        fn set_approval_for_all(
            &mut self,
            operator: Address,
            approved: bool,
        ) -> Result<(), erc721::Error> {
            self.erc721.set_approval_for_all(operator, approved)
        }

        fn get_approved(
            &self,
            token_id: U256,
        ) -> Result<Address, erc721::Error> {
            self.erc721.get_approved(token_id)
        }

        fn is_approved_for_all(
            &self,
            owner: Address,
            operator: Address,
        ) -> bool {
            self.erc721.is_approved_for_all(owner, operator)
        }
    }

    #[public]
    impl IErc721Wrapper for Erc721WrapperTestExample {
        fn underlying(&self) -> Address {
            self.wrapper.underlying()
        }

        fn deposit_for(
            &mut self,
            account: Address,
            token_ids: Vec<U256>,
        ) -> Result<bool, Vec<u8>> {
            self.wrapper.deposit_for(account, token_ids, &mut self.erc721)
        }

        fn withdraw_to(
            &mut self,
            account: Address,
            token_ids: Vec<U256>,
        ) -> Result<bool, Vec<u8>> {
            self.wrapper.withdraw_to(account, token_ids, &mut self.erc721)
        }
    }

    #[public]
    impl IErc721Receiver for Erc721WrapperTestExample {
        fn on_erc721_received(
            &mut self,
            operator: Address,
            from: Address,
            token_id: U256,
            data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            self.wrapper.on_erc721_received(
                operator,
                from,
                token_id,
                &data,
                &mut self.erc721,
            )
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IErc165 for Erc721WrapperTestExample {
        fn supports_interface(&self, interface_id: B32) -> bool {
            self.erc721.supports_interface(interface_id)
        }
    }

    #[motsu::test]
    fn underlying_works(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let erc721_address = erc721_contract.address();

        contract.sender(alice).constructor(erc721_address);

        assert_eq!(contract.sender(alice).underlying(), erc721_address);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "TODO: un-ignore when motsu supports returning empty revert reasons, see: https://github.com/OpenZeppelin/stylus-test-helpers/issues/118"]
    fn deposit_for_reverts_when_underlying_reverts_without_reason(
        contract: Contract<Erc721WrapperTestExample>,
        invalid_underlying: Contract<EmptyReasonReceiver721>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(1);

        let invalid_token = invalid_underlying.address();
        contract.sender(alice).constructor(invalid_token);

        let err = contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect_err("should return Error::UnsupportedToken");

        let expected_error_bytes =
            ERC721UnsupportedToken { token: invalid_token }.abi_encode();
        assert_eq!(err, expected_error_bytes);
    }

    #[motsu::test]
    fn deposit_for_reverts_when_nonexistent_token(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(1);

        contract.sender(alice).constructor(erc721_contract.address());

        let err = contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect_err("should return Error::InvalidReceiverWithReason");

        let expected_error: Vec<u8> =
            erc721::Error::NonexistentToken(erc721::ERC721NonexistentToken {
                token_id: token_ids[0],
            })
            .into();

        assert_eq!(err, expected_error);
    }

    #[motsu::test]
    fn deposit_for_reverts_when_missing_approval(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(1);

        contract.sender(alice).constructor(erc721_contract.address());

        erc721_contract
            .sender(alice)
            ._mint(alice, token_ids[0])
            .motsu_expect("should mint {token_id} for {alice}");

        let err = contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect_err("should return Error::InvalidReceiverWithReason");

        let expected_error: Vec<u8> = erc721::Error::InsufficientApproval(
            erc721::ERC721InsufficientApproval {
                operator: contract.address(),
                token_id: token_ids[0],
            },
        )
        .into();

        assert_eq!(err, expected_error);
    }

    #[motsu::test]
    fn deposit_for_reverts_when_wrapped_token_already_exists(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(1);

        contract.sender(alice).constructor(erc721_contract.address());

        erc721_contract
            .sender(alice)
            ._mint(alice, token_ids[0])
            .motsu_expect("should mint {token_id} for {alice}");

        erc721_contract
            .sender(alice)
            .approve(contract.address(), token_ids[0])
            .motsu_expect("should approve {token_id} for {contract.address()}");

        // Mint an "unexpected" wrapped token.
        contract
            .sender(alice)
            .erc721
            ._mint(alice, token_ids[0])
            .motsu_expect("should mint {token_id} for {alice}");

        let err = contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect_err("should return Error::Erc721");

        let expected_error_bytes =
            erc721::ERC721InvalidSender { sender: Address::ZERO }.abi_encode();
        assert_eq!(err, expected_error_bytes);
    }

    #[motsu::test]
    fn deposit_for_works(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let tokens = 4;
        let token_ids = random_token_ids(tokens);

        contract.sender(alice).constructor(erc721_contract.address());

        for &token_id in &token_ids {
            erc721_contract
                .sender(alice)
                ._mint(alice, token_id)
                .motsu_expect("should mint {token_id} for {alice}");

            erc721_contract
                .sender(alice)
                .approve(contract.address(), token_id)
                .motsu_expect(
                    "should approve {token_id} for {contract.address()}",
                );
        }

        let initial_balance =
            erc721_contract.sender(alice).balance_of(alice).motsu_unwrap();
        let initial_wrapped_balance =
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap();

        let initial_contract_balance = erc721_contract
            .sender(alice)
            .balance_of(contract.address())
            .motsu_unwrap();

        assert!(contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect("should deposit"));

        for token_id in token_ids {
            erc721_contract.assert_emitted(&erc721::Transfer {
                from: alice,
                to: contract.address(),
                token_id,
            });

            contract.assert_emitted(&erc721::Transfer {
                from: Address::ZERO,
                to: alice,
                token_id,
            });
        }

        assert_eq!(
            erc721_contract.sender(alice).balance_of(alice).motsu_unwrap(),
            initial_balance - U256::from(tokens)
        );

        assert_eq!(
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap(),
            initial_wrapped_balance + U256::from(tokens)
        );

        assert_eq!(
            erc721_contract
                .sender(contract.address())
                .balance_of(contract.address())
                .motsu_unwrap(),
            initial_contract_balance + U256::from(tokens)
        );
    }

    #[motsu::test]
    fn withdraw_to_reverts_when_invalid_receiver(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let tokens = 4;
        let token_ids = random_token_ids(tokens);

        contract.sender(alice).constructor(erc721_contract.address());

        for token_id in &token_ids {
            erc721_contract
                .sender(alice)
                ._mint(alice, *token_id)
                .motsu_expect("should mint {token_id} for {alice}");

            erc721_contract
                .sender(alice)
                .approve(contract.address(), *token_id)
                .motsu_expect(
                    "should approve {token_id} for {contract.address()}",
                );
        }

        assert!(contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect("should deposit"));

        let err = contract
            .sender(alice)
            .withdraw_to(Address::ZERO, token_ids.clone())
            .motsu_expect_err("should return Error::InvalidReceiverWithReason");

        let expected_error: Vec<u8> =
            erc721::Error::InvalidReceiver(erc721::ERC721InvalidReceiver {
                receiver: Address::ZERO,
            })
            .into();

        assert_eq!(err, expected_error);
    }

    #[motsu::test]
    fn withdraw_to_reverts_when_nonexistent_token(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let tokens = 1;
        let token_ids = random_token_ids(tokens);

        contract.sender(alice).constructor(erc721_contract.address());

        let err = contract
            .sender(alice)
            .withdraw_to(alice, token_ids.clone())
            .motsu_expect_err("should return Error::Erc721");

        let expected_error_bytes =
            erc721::ERC721NonexistentToken { token_id: token_ids[0] }
                .abi_encode();
        assert_eq!(err, expected_error_bytes);
    }

    #[motsu::test]
    fn withdraw_to_reverts_when_insufficient_approval(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let tokens = 1;
        let token_ids = random_token_ids(tokens);

        contract.sender(alice).constructor(erc721_contract.address());

        erc721_contract
            .sender(alice)
            ._mint(alice, token_ids[0])
            .motsu_expect("should mint {token_id} for {alice}");

        erc721_contract
            .sender(alice)
            .approve(contract.address(), token_ids[0])
            .motsu_expect("should approve {token_id} for {contract.address()}");

        assert!(contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect("should deposit"));

        let err = contract
            .sender(bob)
            .withdraw_to(alice, token_ids.clone())
            .motsu_expect_err("should return Error::Erc721");

        let expected_error_bytes = erc721::ERC721InsufficientApproval {
            token_id: token_ids[0],
            operator: bob,
        }
        .abi_encode();
        assert_eq!(err, expected_error_bytes);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "TODO: un-ignore when motsu supports returning empty revert reasons, see: https://github.com/OpenZeppelin/stylus-test-helpers/issues/118"]
    fn withdraw_to_reverts_when_underlying_reverts_without_reason(
        contract: Contract<Erc721WrapperTestExample>,
        invalid_underlying: Contract<EmptyReasonReceiver721>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(1);

        contract.sender(alice).constructor(invalid_underlying.address());

        for &token_id in &token_ids {
            contract
                .sender(alice)
                .erc721
                ._mint(alice, token_id)
                .motsu_expect("should mint {token_id} for {alice}");

            contract
                .sender(alice)
                .approve(contract.address(), token_id)
                .motsu_expect(
                    "should approve {token_id} for {contract.address()}",
                );
        }

        let err = contract
            .sender(alice)
            .withdraw_to(alice, token_ids.clone())
            .motsu_expect_err("should return empty reason");

        assert_eq!(
            err,
            Erc721FailedOperation { token: invalid_underlying.address() }
                .abi_encode()
        );
    }

    #[motsu::test]
    fn withdraw_to_works(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let tokens = 4;
        let token_ids = random_token_ids(tokens);

        contract.sender(alice).constructor(erc721_contract.address());

        for token_id in &token_ids {
            erc721_contract
                .sender(alice)
                ._mint(alice, *token_id)
                .motsu_expect("should mint {token_id} for {alice}");

            erc721_contract
                .sender(alice)
                .approve(contract.address(), *token_id)
                .motsu_expect(
                    "should approve {token_id} for {contract.address()}",
                );
        }

        assert!(contract
            .sender(alice)
            .deposit_for(alice, token_ids.clone())
            .motsu_expect("should deposit"));

        let initial_balance =
            erc721_contract.sender(alice).balance_of(alice).motsu_unwrap();
        let initial_wrapped_balance =
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap();

        let initial_contract_balance = erc721_contract
            .sender(alice)
            .balance_of(contract.address())
            .motsu_unwrap();

        assert!(contract
            .sender(alice)
            .withdraw_to(alice, token_ids.clone())
            .motsu_expect("should withdraw"));

        for token_id in token_ids {
            erc721_contract.assert_emitted(&erc721::Transfer {
                from: contract.address(),
                to: alice,
                token_id,
            });

            contract.assert_emitted(&erc721::Transfer {
                from: alice,
                to: Address::ZERO,
                token_id,
            });
        }

        assert_eq!(
            erc721_contract.sender(alice).balance_of(alice).motsu_unwrap(),
            initial_balance + U256::from(tokens)
        );

        assert_eq!(
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap(),
            initial_wrapped_balance - U256::from(tokens)
        );

        assert_eq!(
            erc721_contract
                .sender(contract.address())
                .balance_of(contract.address())
                .motsu_unwrap(),
            initial_contract_balance - U256::from(tokens)
        );
    }

    #[motsu::test]
    fn on_erc721_received_reverts_when_sender_is_unsupported_token(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];

        contract.sender(alice).constructor(erc721_contract.address());

        let invalid_operator = alice;

        let err = contract
            .sender(invalid_operator)
            .on_erc721_received(
                invalid_operator,
                alice,
                token_id,
                vec![].into(),
            )
            .motsu_expect_err("should return Error::UnsupportedToken");

        assert_eq!(
            err,
            ERC721UnsupportedToken { token: invalid_operator }.abi_encode()
        );
    }

    #[motsu::test]
    fn on_erc721_received_reverts_when_wrapped_token_already_exists(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];

        contract.sender(alice).constructor(erc721_contract.address());

        // Mint an "unexpected" wrapped token.
        contract
            .sender(alice)
            .erc721
            ._mint(alice, token_id)
            .motsu_expect("should mint {token_id} for {alice}");

        let operator = alice;

        let err = contract
            .sender(erc721_contract.address())
            .on_erc721_received(operator, alice, token_id, vec![].into())
            .motsu_expect_err("should return Error::Erc721");

        assert_eq!(
            err,
            erc721::ERC721InvalidSender { sender: Address::ZERO }.abi_encode()
        );
    }

    #[motsu::test]
    fn on_erc721_received_works(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];

        contract.sender(alice).constructor(erc721_contract.address());

        let initial_wrapped_balance =
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap();

        let operator = alice;
        let interface_id = contract
            .sender(erc721_contract.address())
            .on_erc721_received(operator, alice, token_id, vec![].into())
            .motsu_expect("should handle ERC721Received");

        assert_eq!(interface_id, RECEIVER_FN_SELECTOR);

        contract.assert_emitted(&erc721::Transfer {
            from: Address::ZERO,
            to: alice,
            token_id,
        });

        assert_eq!(
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap(),
            initial_wrapped_balance + U256::ONE
        );
    }

    #[storage]
    struct InvalidToken;

    unsafe impl TopLevelStorage for InvalidToken {}

    #[public]
    #[allow(clippy::unused_self)]
    impl InvalidToken {
        fn owner_of(&self, _token_id: U256) -> Result<Address, Vec<u8>> {
            Err("InvalidToken".into())
        }
    }

    #[motsu::test]
    #[ignore = "impossible with current motsu limitations"]
    fn recover_reverts_when_invalid_token(
        contract: Contract<Erc721WrapperTestExample>,
        invalid_token: Contract<InvalidToken>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];

        contract.sender(alice).constructor(invalid_token.address());

        let err = contract
            .sender(alice)
            .recover(alice, token_id)
            .motsu_expect_err("should return Error::Erc721FailedOperation");

        assert!(matches!(
            err,
            Error::Erc721FailedOperation(Erc721FailedOperation { token })
                if token == invalid_token.address()
        ));
    }

    #[motsu::test]
    fn recover_reverts_when_incorrect_owner(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];

        contract.sender(alice).constructor(erc721_contract.address());

        erc721_contract
            .sender(alice)
            ._mint(alice, token_id)
            .motsu_expect("should mint {token_id} for {alice}");

        let err = contract
            .sender(alice)
            .recover(alice, token_id)
            .motsu_expect_err("should return Error::Erc721");

        assert!(matches!(
            err,
            Error::IncorrectOwner(
                erc721::ERC721IncorrectOwner { sender, token_id: t_id, owner },
            ) if sender == contract.address() && t_id == token_id && owner == alice
        ));
    }

    #[motsu::test]
    fn recover_reverts_when_wrapped_token_already_exists(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];

        contract.sender(alice).constructor(erc721_contract.address());

        erc721_contract
            .sender(alice)
            ._mint(alice, token_id)
            .motsu_expect("should mint {token_id} for {alice}");

        erc721_contract
            .sender(alice)
            .transfer_from(
                alice,
                contract.address(),
                token_id,
            )
            .motsu_expect("should transfer {token_id} from {alice} to {contract.address()}");

        // Mint an "unexpected" wrapped token.
        contract
            .sender(alice)
            .erc721
            ._mint(alice, token_id)
            .motsu_expect("should mint {token_id} for {alice}");

        let err = contract
            .sender(alice)
            .recover(alice, token_id)
            .motsu_expect_err("should return Error::Erc721");

        assert!(matches!(
            err,
            Error::InvalidSender(
                erc721::ERC721InvalidSender { sender }
            ) if sender.is_zero()
        ));
    }

    #[motsu::test]
    fn recover_works(
        contract: Contract<Erc721WrapperTestExample>,
        erc721_contract: Contract<Erc721>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];

        contract.sender(alice).constructor(erc721_contract.address());

        erc721_contract
            .sender(alice)
            ._mint(alice, token_id)
            .motsu_expect("should mint {token_id} for {alice}");

        erc721_contract
            .sender(alice)
            .transfer_from(
                alice,
                contract.address(),
                token_id,
            )
            .motsu_expect("should transfer {token_id} from {alice} to {contract.address()}");

        let initial_wrapped_balance =
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap();

        contract
            .sender(alice)
            .recover(alice, token_id)
            .motsu_expect("should recover {token_id} for {alice}");

        let wrapped_balance =
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap();

        assert_eq!(wrapped_balance, initial_wrapped_balance + U256::ONE);

        contract.assert_emitted(&erc721::Transfer {
            from: Address::ZERO,
            to: alice,
            token_id,
        });

        assert_eq!(
            contract.sender(alice).erc721.balance_of(alice).motsu_unwrap(),
            initial_wrapped_balance + U256::ONE
        );
    }
}
