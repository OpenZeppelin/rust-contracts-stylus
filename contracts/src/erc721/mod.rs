use std::borrow::BorrowMut;
use std::prelude::v1::{Box, String, ToString, Vec};
use std::vec;
use derive_more::From;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    call::Call,
    evm, msg,
    prelude::*,
};

sol! {
    /// Emitted when `tokenId` token is transferred from `from` to `to`.
    ///
    /// * `from` - Address from which token will be transferred.
    /// * `to` - Address where token will be transferred.
    /// * `token_id` - Token id as a number.
    event Transfer(address indexed from, address indexed to, uint256 indexed token_id);

    /// Emitted when `owner` enables `approved` to manage the `tokenId` token.
    ///
    /// * `owner` - Address of the owner of the token.
    /// * `approved` - Address of the approver.
    /// * `token_id` - Token id as a number.
    event Approval(address indexed owner, address indexed approved, uint256 indexed token_id);

    /// Emitted when `owner` enables or disables (`approved`) `operator` to manage all of its assets.
    ///
    /// * `owner` - Address of the owner of the token.
    /// * `operator` - Address of an operator that will manage operations on the token.
    /// * `token_id` - Token id as a number.
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
}

sol! {
    /// Indicates that an address can't be an owner.
    /// For example, `address(0)` is a forbidden owner in ERC-20. Used in balance queries.
    ///
    /// * `owner` - Incorrect address of the owner.
    #[derive(Debug)]
    error ERC721InvalidOwner(address owner);

    /// Indicates a `tokenId` whose `owner` is the zero address.
    ///
    /// * `token_id` - Token id as a number.
    #[derive(Debug)]
    error ERC721NonexistentToken(uint256 token_id);

    /// Indicates an error related to the ownership over a particular token. Used in transfers.
    ///
    /// * `sender` - Address whose token being transferred.
    /// * `token_id` - Token id as a number.
    /// * `owner` - Address of the owner of the token.
    #[derive(Debug)]
    error ERC721IncorrectOwner(address sender, uint256 token_id, address owner);

    /// Indicates a failure with the token `sender`. Used in transfers.
    ///
    /// * `sender` - An address whose token being transferred.
    #[derive(Debug)]
    error ERC721InvalidSender(address sender);

    /// Indicates a failure with the token `receiver`. Used in transfers.
    ///
    /// * `receiver` - Address that receives token.
    #[derive(Debug)]
    error ERC721InvalidReceiver(address receiver);

    /// Indicates a failure with the `operator`’s approval. Used in transfers.
    ///
    /// * `address` - Address of an operator that wasn't approved.
    /// * `token_id` - Token id as a number.
    #[derive(Debug)]
    error ERC721InsufficientApproval(address operator, uint256 token_id);

    /// Indicates a failure with the `approver` of a token to be approved. Used in approvals.
    ///
    /// * `address` - Address of an approver that failed to approve.
    #[derive(Debug)]
    error ERC721InvalidApprover(address approver);

    /// Indicates a failure with the `operator` to be approved. Used in approvals.
    #[derive(Debug)]
    /// * `operator` - Incorrect address of the operator.
    error ERC721InvalidOperator(address operator);
}

/// An ERC-721 error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug, From)]
pub enum Error {
    ERC721InvalidOwner(ERC721InvalidOwner),
    ERC721NonexistentToken(ERC721NonexistentToken),
    ERC721IncorrectOwner(ERC721IncorrectOwner),
    ERC721InvalidSender(ERC721InvalidSender),
    ERC721InvalidReceiver(ERC721InvalidReceiver),
    ERC721InsufficientApproval(ERC721InsufficientApproval),
    ERC721InvalidApprover(ERC721InvalidApprover),
    ERC721InvalidOperator(ERC721InvalidOperator),
}

sol_interface! {
    /// ERC-721 token receiver interface.
    /// Interface for any contract that wants to support safeTransfers
    /// from ERC-721 asset contracts.
    interface IERC721Receiver {
        /// Whenever an {IERC721} `tokenId` token is transferred to this contract via {IERC721-safeTransferFrom}
        /// by `operator` from `from`, this function is called.
        ///
        /// It must return its Solidity selector to confirm the token transfer.
        /// If any other value is returned or the interface is not implemented by the recipient, the transfer will be
        /// reverted.
        ///
        /// The selector can be obtained in Solidity with `IERC721Receiver.onERC721Received.selector`.
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);
    }
}

sol_storage! {
    pub struct Erc721 {
        mapping(uint256 => address) owners;

        mapping(address => uint256) balances;

        mapping(uint256 => address) token_approvals;

        mapping(address => mapping(address => bool)) operator_approvals;
    }
}

#[external]
impl Erc721 {
    /// Returns the number of tokens in ``owner``'s account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    ///
    /// # Returns
    ///
    /// The balance of the owner.
    pub fn balance_of(&self, owner: Address) -> Result<U256, Error> {
        if owner == Address::ZERO {
            return Err(ERC721InvalidOwner { owner: Address::ZERO }.into());
        }
        Ok(self.balances.get(owner))
    }

    /// Returns the owner of the `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number
    ///
    /// # Returns
    ///
    /// The owner of the token.
    ///
    /// # Requirements
    ///
    /// * `token_id` must exist.
    pub fn owner_of(&self, token_id: U256) -> Result<Address, Error> {
        self.require_owned(token_id)
    }

    /// Safely transfers `token_id` token from `from` to `to`, checking first that contract recipients
    /// are aware of the ERC-721 protocol to prevent tokens from being forever locked.
    ///
    /// # Arguments
    ///
    /// * `storage` - Write access to the contract's state.
    /// * `from` - Account of the sender
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    ///
    /// # Returns
    ///
    /// Result indicating success or failure.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * `token_id` token must exist and be owned by `from`.
    /// * If the caller is not `from`, it must have been allowed to move this token by either {approve} or
    /// * {setApprovalForAll}.
    /// * If `to` refers to a smart contract, it must implement {IERC721Receiver-onERC721Received}, which is called upon
    /// * a safe transfer.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn safe_transfer_from(
        storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        Self::safe_transfer_from_with_data(
            storage,
            from,
            to,
            token_id,
            vec![].into(),
        )
    }

    /// Safely transfers `token_id` token from `from` to `to`.
    ///
    /// # Arguments
    ///
    /// * `storage` - Write access to the contract's state.
    /// * `from` - Account of the sender
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    /// * `data` - Additional data with no specified format, sent in call to `to`
    ///
    /// # Returns
    ///
    /// Result indicating success or failure.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * `token_id` token must exist and be owned by `from`.
    /// * If the caller is not `from`, it must be approved to move this token by either {approve} or [`set_approval_for_all`].
    /// * If `to` refers to a smart contract, it must implement {IERC721Receiver-onERC721Received}, which is called upon
    /// * a safe transfer.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data(
        storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        storage.borrow_mut().transfer_from(from, to, token_id)?;
        Self::check_on_erc721_received(
            storage,
            msg::sender(),
            from,
            to,
            token_id,
            data,
        )
    }

    /// Transfers `token_id` token from `from` to `to`.
    ///
    /// WARNING: Note that the caller is responsible to confirm that the recipient is capable of receiving ERC-721
    /// or else they may be permanently lost. Usage of {safe_transfer_from} prevents loss, though the caller must
    /// understand this adds an external call which potentially creates a reentrancy vulnerability.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    ///
    /// # Requirements:
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * `token_id` token must be owned by `from`.
    /// * If the caller is not `from`, it must be approved to move this token by either {approve} or [`set_approval_for_all`].
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to == Address::ZERO {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        // Setting an "auth" arguments enables the `_isAuthorized` check which verifies that the token exists
        // (from != 0). Therefore, it is not needed to verify that the return value is not 0 here.
        let previous_owner = self.update(to, token_id, msg::sender())?;
        if previous_owner != from {
            return Err(ERC721IncorrectOwner {
                sender: from,
                token_id: token_id,
                owner: previous_owner,
            }
            .into());
        }
        Ok(())
    }

    /// Gives permission to `to` to transfer `token_id` token to another account.
    /// The approval is cleared when the token is transferred.
    ///
    /// Only a single account can be approved at a time, so approving the zero address clears previous approvals.
    ///
    /// # Arguments
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    ///
    /// # Requirements:
    ///
    /// - The caller must own the token or be an approved operator.
    /// - `token_id` must exist.
    ///
    /// # Events
    ///
    /// Emits an [`Approval`] event.
    pub fn approve(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        self.approve_inner(to, token_id, msg::sender(), true)
    }

    /// Approve or remove `operator` as an operator for the caller.
    /// Operators can call {transfer_from} or {safe_transfer_from} for any token owned by the caller.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account add to the set of authorized operators.
    /// * `approved` - Flag that that set approval or disapproval for the operator.
    ///
    /// # Requirements:
    ///
    /// * The `operator` cannot be the address zero.
    ///
    /// # Events
    ///
    /// Emits an [`ApprovalForAll`] event.
    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        self.set_approval_for_all_inner(msg::sender(), operator, approved)
    }

    /// Returns the account approved for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    pub fn get_approved(&self, token_id: U256) -> Result<Address, Error> {
        self.require_owned(token_id)?;
        self.get_approved_inner(token_id)
    }

    /// Returns if the `operator` is allowed to manage all the assets of `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `operator` - Account to add to the set of authorized operators.
    ///
    /// # Events
    ///
    /// Emits an [`set_approval_for_all`] event
    pub fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, Error> {
        Ok(self.operator_approvals.get(owner).get(operator))
    }
}

impl Erc721 {
    /// Returns the owner of the `token_id`. Does NOT revert if token doesn't exist
    ///
    /// IMPORTANT: Any overrides to this function that add ownership of tokens not tracked by the
    /// core ERC-721 logic MUST be matched with the use of {_increaseBalance} to keep balances
    /// consistent with ownership. The invariant to preserve is that for any address `a` the value returned by
    /// `balance_of(a)` must be equal to the number of tokens such that `owner_of_inner(token_id)` is `a`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number
    pub fn owner_of_inner(&self, token_id: U256) -> Result<Address, Error> {
        Ok(self.owners.get(token_id))
    }

    /// Returns the approved address for `token_id`. Returns 0 if `token_id` is not minted.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number
    pub fn get_approved_inner(&self, token_id: U256) -> Result<Address, Error> {
        Ok(self.token_approvals.get(token_id))
    }

    /// Returns whether `spender` is allowed to manage `owner`'s tokens, or `token_id` in
    /// particular (ignoring whether it is owned by `owner`).
    ///
    /// WARNING: This function assumes that `owner` is the actual owner of `token_id` and does not verify this
    /// assumption.
    ///
    /// # Arguments
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `spender` - Account that will spend token.
    /// * `token_id` - Token id as a number
    pub fn is_authorized(
        &self,
        owner: Address,
        spender: Address,
        token_id: U256,
    ) -> Result<bool, Error> {
        let is_authorized = spender != Address::ZERO
            && (owner == spender
                || self.is_approved_for_all(owner, spender)?
                || self.get_approved_inner(token_id)? == spender);
        Ok(is_authorized)
    }

    /// Checks if `spender` can operate on `token_id`, assuming the provided `owner` is the actual owner.
    /// Reverts if `spender` does not have approval from the provided `owner` for the given token or for all its assets
    /// the `spender` for the specific `token_id`.
    ///
    /// WARNING: This function assumes that `owner` is the actual owner of `token_id` and does not verify this
    /// assumption.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `spender` - Account that will spend token.
    /// * `token_id` - Token id as a number
    pub fn check_authorized(
        &self,
        owner: Address,
        spender: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if !self.is_authorized(owner, spender, token_id)? {
            return if owner == Address::ZERO {
                Err(ERC721NonexistentToken { token_id: token_id }.into())
            } else {
                Err(ERC721InsufficientApproval {
                    operator: spender,
                    token_id: token_id,
                }
                .into())
            };
        }
        Ok(())
    }

    /// Unsafe write access to the balances, used by extensions that "mint" tokens using an {owner_of} override.
    ///
    /// NOTE: the value is limited to type(uint128).max. This protect against _balance overflow. It is unrealistic that
    /// a uint256 would ever overflow from increments when these increments are bounded to uint128 values.
    ///
    /// WARNING: Increasing an account's balance using this function tends to be paired with an override of the
    /// {owner_of_inner} function to resolve the ownership of the corresponding tokens so that balances and ownership
    /// remain consistent with one another.
    ///
    /// # Arguments    
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - Account to increase balance.
    /// * `value` - The number of tokens to increase balance.
    pub fn increase_balance(&mut self, account: Address, value: U256) {
        self.balances.setter(account).add_assign_unchecked(value);
    }

    /// Transfers `token_id` from its current owner to `to`, or alternatively mints (or burns) if the current owner
    /// (or `to`) is the zero address. Returns the owner of the `token_id` before the update.
    ///
    /// The `auth` argument is optional. If the value passed is non 0, then this function will check that
    /// `auth` is either the owner of the token, or approved to operate on the token (by the owner).
    ///
    /// NOTE: If overriding this function in a way that tracks balances, see also {_increaseBalance}.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `auth` - Account used for authorization of the update.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn update(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let from = self.owner_of_inner(token_id)?;

        // Perform (optional) operator check
        if auth != Address::ZERO {
            self.check_authorized(from, auth, token_id)?;
        }

        // Execute the update
        if from != Address::ZERO {
            // Clear approval. No need to re-authorize or emit the Approval event
            self.approve_inner(Address::ZERO, token_id, Address::ZERO, false)?;
            self.balances.setter(from).sub_assign_unchecked(U256::from(1));
        }

        if to != Address::ZERO {
            self.balances.setter(to).add_assign_unchecked(U256::from(1))
        }

        self.owners.setter(token_id).set(to);

        evm::log(Transfer { from, to, token_id: token_id });

        Ok(from)
    }

    /// Mints `token_id` and transfers it to `to`.
    ///
    /// WARNING: Usage of this method is discouraged, use {safe_mint} whenever possible
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    ///
    /// # Requirements:
    ///
    /// * `token_id` must not exist.
    /// * `to` cannot be the zero address.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        if to == Address::ZERO {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        let previous_owner = self.update(to, token_id, Address::ZERO)?;
        if previous_owner != Address::ZERO {
            return Err(ERC721InvalidSender { sender: Address::ZERO }.into());
        }
        Ok(())
    }

    /// Same as {xref-ERC721-safe_mint-address-uint256-}[`_safeMint`], with an additional `data` parameter which is
    /// forwarded in {IERC721Receiver-onERC721Received} to contract recipients.
    ///
    /// # Arguments
    ///
    /// * `storage` - Write access to the contract's state.
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    /// * `data` - Additional data with no specified format, sent in call to `to`
    pub fn safe_mint(
        storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        storage.borrow_mut().mint(to, token_id)?;
        Self::check_on_erc721_received(
            storage,
            msg::sender(),
            Address::ZERO,
            to,
            token_id,
            data,
        )
    }

    /// Destroys `token_id`.
    /// The approval is cleared when the token is burned.
    /// This is an internal function that does not check if the sender is authorized to operate on the token.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Token id as a number
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        let previous_owner =
            self.update(Address::ZERO, token_id, Address::ZERO)?;
        if previous_owner == Address::ZERO {
            Err(ERC721NonexistentToken { token_id: token_id }.into())
        } else {
            Ok(())
        }
    }

    /// Transfers `token_id` from `from` to `to`.
    ///  As opposed to {transferFrom}, this imposes no restrictions on msg.sender.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    ///
    /// # Requirements:
    ///
    /// * `to` cannot be the zero address.
    /// * `token_id` token must be owned by `from`.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to == Address::ZERO {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        let previous_owner = self.update(to, token_id, Address::ZERO)?;
        if previous_owner == Address::ZERO {
            Err(ERC721NonexistentToken { token_id: token_id }.into())
        } else if previous_owner != from {
            Err(ERC721IncorrectOwner {
                sender: from,
                token_id: token_id,
                owner: previous_owner,
            }
            .into())
        } else {
            Ok(())
        }
    }

    /// Same as {xref-ERC721-safe_transfer-address-address-uint256-}[`_safeTransfer`], with an additional `data` parameter which is
    /// forwarded in {IERC721Receiver-onERC721Received} to contract recipients.
    ///
    /// # Arguments
    ///
    /// * `storage` - Write access to the contract's state.
    /// * `from` - Account of the sender
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    /// * `data` - Additional data with no specified format, sent in call to `to`
    pub fn safe_transfer(
        storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        storage.borrow_mut().transfer(from, to, token_id)?;
        Self::check_on_erc721_received(
            storage,
            msg::sender(),
            from,
            to,
            token_id,
            data,
        )
    }

    /// Variant of `approve_inner` with an optional flag to enable or disable the {Approval} event. The event is not
    /// emitted in the context of transfers.
    ///
    /// # Arguments
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    /// * `auth` - Account used for authorization of the update.
    /// * `emit_event` - Emit ['Approval'] event flag.
    pub fn approve_inner(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
        emit_event: bool,
    ) -> Result<(), Error> {
        // Avoid reading the owner unless necessary
        if emit_event || auth != Address::ZERO {
            let owner = self.require_owned(token_id)?;

            // We do not use _isAuthorized because single-token approvals should not be able to call approve
            if auth != Address::ZERO
                && owner != auth
                && !self.is_approved_for_all(owner, auth)?
            {
                return Err(ERC721InvalidApprover { approver: auth }.into());
            }

            if emit_event {
                evm::log(Approval { owner, approved: to, token_id: token_id });
            }
        }

        self.token_approvals.setter(token_id).set(to);
        Ok(())
    }

    /// Approve `operator` to operate on all of `owner` tokens
    ///
    /// # Arguments
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account the token's owner.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `approved` - Flag that that set approval or disapproval for the operator.
    ///
    /// # Requirements:
    /// * operator can't be the address zero.
    ///
    /// # Events
    ///
    /// Emits an {ApprovalForAll} event.
    pub fn set_approval_for_all_inner(
        &mut self,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        if operator == Address::ZERO {
            return Err(ERC721InvalidOperator { operator }.into());
        }
        self.operator_approvals.setter(owner).setter(operator).set(approved);
        evm::log(ApprovalForAll { owner, operator, approved });
        Ok(())
    }

    /// Reverts if the `token_id` doesn't have a current owner (it hasn't been minted, or it has been burned).
    /// Returns the owner.
    ///
    /// Overrides to ownership logic should be done to {owner_of_inner}.
    ///
    /// # Arguments
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number
    pub fn require_owned(&self, token_id: U256) -> Result<Address, Error> {
        let owner = self.owner_of_inner(token_id)?;
        if owner == Address::ZERO {
            return Err(ERC721NonexistentToken { token_id: token_id }.into());
        }
        Ok(owner)
    }

    /// Performs an acceptance check for the provided `operator` by calling {IERC721-onERC721Received}
    /// on the `to` address. The `operator` is generally the address that initiated the token transfer (i.e. `msg.sender`).
    ///
    /// The acceptance call is not executed and treated as a no-op if the target address is doesn't contain code (i.e. an EOA).
    /// Otherwise, the recipient must implement {IERC721Receiver-onERC721Received} and return the acceptance magic value to accept
    /// the transfer.
    ///
    /// # Arguments
    /// * `storage` - Write access to the contract's state.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `from` - Account of the sender
    /// * `to` - Account of the recipient
    /// * `token_id` - Token id as a number
    /// * `data` - Additional data with no specified format, sent in call to `to`
    pub fn check_on_erc721_received(
        storage: &mut impl TopLevelStorage,
        operator: Address,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        // TODO: compute INTERFACE_ID at compile time
        const IERC721RECEIVER_INTERFACE_ID: u32 = 0x150b7a02;
        if to.has_code() {
            let call = Call::new_in(storage);
            return match IERC721Receiver::new(to).on_erc_721_received(
                call,
                operator,
                from,
                token_id,
                data.to_vec(),
            ) {
                Ok(result) => {
                    let received_interface_id = u32::from_be_bytes(result.0);
                    if received_interface_id != IERC721RECEIVER_INTERFACE_ID {
                        Err(ERC721InvalidReceiver { receiver: to }.into())
                    } else {
                        Ok(())
                    }
                }
                Err(_) => Err(ERC721InvalidReceiver{receiver: to}.into()),
            };
        }
        Ok(())
    }
}

use stylus_sdk::storage::{StorageGuardMut, StorageMap, StorageUint};

pub trait IncrementalMath<T> {
    fn add_assign_unchecked(&mut self, rhs: T);

    fn sub_assign_unchecked(&mut self, rhs: T);
}

impl<'a> IncrementalMath<U256> for StorageGuardMut<'a, StorageUint<256, 4>> {
    fn add_assign_unchecked(&mut self, rhs: U256) {
        let new_balance = self.get() + rhs;
        self.set(new_balance);
    }

    fn sub_assign_unchecked(&mut self, rhs: U256) {
        let new_balance = self.get() - rhs;
        self.set(new_balance);
    }
}

#[cfg(test)]
mod tests {
    use crate::erc721::{Erc721, Error};
    #[allow(unused_imports)]
    use crate::test_utils;
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::{
        msg,
        storage::{StorageMap, StorageType, StorageU256},
    };

    impl Default for Erc721 {
        fn default() -> Self {
            let root = U256::ZERO;

            Erc721 {
                owners: unsafe { StorageMap::new(root, 0) },
                balances: unsafe { StorageMap::new(root + U256::from(32), 0) },
                token_approvals: unsafe {
                    StorageMap::new(root + U256::from(64), 0)
                },
                operator_approvals: unsafe {
                    StorageMap::new(root + U256::from(96), 0)
                },
            }
        }
    }

    #[test]
    fn reads_balance() {
        test_utils::with_storage::<Erc721>(|token| {
            // TODO#q create random address
            let address = address!("01fA6bf4Ee48B6C95900BCcf9BEA172EF5DBd478");
            let balance = token.balance_of(address);
            assert_eq!(U256::ZERO, balance.unwrap());

            let owner = msg::sender();
            let one = U256::from(1);
            token.balances.setter(owner).set(one);
            let balance = token.balance_of(owner);
            assert_eq!(one, balance.unwrap());
        });
    }
}
