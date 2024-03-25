use std::borrow::BorrowMut;
use std::marker::PhantomData;
use std::prelude::v1::{String, ToString, Vec};
use std::vec;
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    call::Call,
    evm, function_selector, msg,
    prelude::*,
};

sol_storage! {
    pub struct Erc721<T> {
        mapping(uint256 => address) owners;

        mapping(address => uint256) balances;

        mapping(uint256 => address) token_approvals;

        mapping(address => mapping(address => bool)) operator_approvals;

        PhantomData<T> phantom_data;
    }
}

sol! {
    /// Emitted when `tokenId` token is transferred from `from` to `to`.
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    
    /// Emitted when `owner` enables `approved` to manage the `tokenId` token.
    event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
    
    /// Emitted when `owner` enables or disables (`approved`) `operator` to manage all of its assets.
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
}

sol! {
    /// Indicates that an address can't be an owner.
    /// For example, `address(0)` is a forbidden owner in ERC-20. Used in balance queries.
    error ERC721InvalidOwner(address owner);

    /// Indicates a `tokenId` whose `owner` is the zero address.
    error ERC721NonexistentToken(uint256 tokenId);

    /// Indicates an error related to the ownership over a particular token. Used in transfers.
    error ERC721IncorrectOwner(address sender, uint256 tokenId, address owner);

    /// Indicates a failure with the token `sender`. Used in transfers.
    error ERC721InvalidSender(address sender);

    /// Indicates a failure with the token `receiver`. Used in transfers.
    error ERC721InvalidReceiver(address receiver);

    /// Indicates a failure with the `operator`’s approval. Used in transfers.
    error ERC721InsufficientApproval(address operator, uint256 tokenId);

    /// Indicates a failure with the `approver` of a token to be approved. Used in approvals.
    error ERC721InvalidApprover(address approver);

    /// Indicates a failure with the `operator` to be approved. Used in approvals.
    error ERC721InvalidOperator(address operator);
}

sol_interface! {
    /// ERC-721 token receiver interface
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
            uint256 tokenId,
            bytes calldata data
        ) external returns (bytes4);
    }
}

pub trait Erc721Info {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const BASE_URI: &'static str;
}

pub struct Erc721Error(Vec<u8>);

// NOTE: According to current implementation of stylus every error should be converted to Vec<u8>
impl From<Erc721Error> for Vec<u8> {
    fn from(value: Erc721Error) -> Vec<u8> {
        value.0
    }
}

impl<T: SolError> From<T> for Erc721Error {
    fn from(value: T) -> Self {
        Self(value.encode())
    }
}

#[external]
impl<T: Erc721Info> Erc721<T> {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    #[selector(name = "tokenURI")]
    pub fn token_uri(token_id: U256) -> Result<String, Erc721Error> {
        let token_uri = if T::BASE_URI.is_empty() {
            "".to_string()
        } else {
            T::BASE_URI.to_string() + &token_id.to_string()
        };
        Ok(token_uri)
    }

    /// Returns the number of tokens in ``owner``'s account.
    pub fn balance_of(&self, owner: Address) -> Result<U256, Erc721Error> {
        if owner == Address::ZERO {
            return Err(ERC721InvalidOwner { owner: Address::ZERO }.into());
        }
        Ok(self.balances.get(owner))
    }

    /// Returns the owner of the `token_id` token.
    ///
    /// Requirements:
    ///
    /// - `token_id` must exist.
    pub fn owner_of(&self, token_id: U256) -> Result<Address, Erc721Error> {
        self.require_owned(token_id)
    }

    /// Safely transfers `token_id` token from `from` to `to`, checking first that contract recipients
    /// are aware of the ERC-721 protocol to prevent tokens from being forever locked.
    ///
    /// Requirements:
    ///
    /// - `from` cannot be the zero address.
    /// - `to` cannot be the zero address.
    /// - `token_id` token must exist and be owned by `from`.
    /// - If the caller is not `from`, it must have been allowed to move this token by either {approve} or
    ///   {setApprovalForAll}.
    /// - If `to` refers to a smart contract, it must implement {IERC721Receiver-onERC721Received}, which is called upon
    ///   a safe transfer.
    ///
    /// Emits a {Transfer} event.
    pub fn safe_transfer_from(
        toplevel_storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        Self::safe_transfer_from_with_data(
            toplevel_storage,
            from,
            to,
            token_id,
            vec![].into(),
        )
    }

    /// Safely transfers `token_id` token from `from` to `to`.
    ///
    /// Requirements:
    ///
    /// - `from` cannot be the zero address.
    /// - `to` cannot be the zero address.
    /// - `token_id` token must exist and be owned by `from`.
    /// - If the caller is not `from`, it must be approved to move this token by either {approve} or {set_approval_for_all}.
    /// - If `to` refers to a smart contract, it must implement {IERC721Receiver-onERC721Received}, which is called upon
    ///   a safe transfer.
    ///
    /// Emits a {Transfer} event.
    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data(
        toplevel_storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Erc721Error> {
        toplevel_storage.borrow_mut().transfer_from(from, to, token_id)?;
        Self::check_on_erc721_received(
            toplevel_storage,
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
    /// Requirements:
    ///
    /// - `from` cannot be the zero address.
    /// - `to` cannot be the zero address.
    /// - `token_id` token must be owned by `from`.
    /// - If the caller is not `from`, it must be approved to move this token by either {approve} or {set_approval_for_all}.
    ///
    /// Emits a {Transfer} event.
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
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
                tokenId: token_id,
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
    /// Requirements:
    ///
    /// - The caller must own the token or be an approved operator.
    /// - `token_id` must exist.
    ///
    /// Emits an {Approval} event.
    pub fn approve(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        self.approve_inner(to, token_id, msg::sender(), true)
    }

    /// Approve or remove `operator` as an operator for the caller.
    /// Operators can call {transfer_from} or {safe_transfer_from} for any token owned by the caller.
    ///
    /// Requirements:
    ///
    /// - The `operator` cannot be the address zero.
    ///
    /// Emits an {ApprovalForAll} event.
    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Erc721Error> {
        self.set_approval_for_all_inner(msg::sender(), operator, approved)
    }

    /// Returns the account approved for `token_id` token.
    ///
    /// Requirements:
    ///
    /// - `token_id` must exist.
    pub fn get_approved(&self, token_id: U256) -> Result<Address, Erc721Error> {
        self.require_owned(token_id)?;
        self.get_approved_inner(token_id)
    }

    /// Returns if the `operator` is allowed to manage all the assets of `owner`.
    ///
    /// See {set_approval_for_all}
    pub fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, Erc721Error> {
        Ok(self.operator_approvals.get(owner).get(operator))
    }
}

impl<T: Erc721Info> Erc721<T> {
    /// Returns the owner of the `token_id`. Does NOT revert if token doesn't exist
    ///
    /// IMPORTANT: Any overrides to this function that add ownership of tokens not tracked by the
    /// core ERC-721 logic MUST be matched with the use of {_increaseBalance} to keep balances
    /// consistent with ownership. The invariant to preserve is that for any address `a` the value returned by
    /// `balance_of(a)` must be equal to the number of tokens such that `owner_of_inner(token_id)` is `a`.
    pub fn owner_of_inner(
        &self,
        token_id: U256,
    ) -> Result<Address, Erc721Error> {
        Ok(self.owners.get(token_id))
    }

    /// Returns the approved address for `token_id`. Returns 0 if `token_id` is not minted.
    pub fn get_approved_inner(
        &self,
        token_id: U256,
    ) -> Result<Address, Erc721Error> {
        Ok(self.token_approvals.get(token_id))
    }

    /// Returns whether `spender` is allowed to manage `owner`'s tokens, or `token_id` in
    /// particular (ignoring whether it is owned by `owner`).
    ///
    /// WARNING: This function assumes that `owner` is the actual owner of `token_id` and does not verify this
    /// assumption.
    pub fn is_authorized(
        &self,
        owner: Address,
        spender: Address,
        token_id: U256,
    ) -> Result<bool, Erc721Error> {
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
    pub fn check_authorized(
        &self,
        owner: Address,
        spender: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        if !self.is_authorized(owner, spender, token_id)? {
            return if owner == Address::ZERO {
                Err(ERC721NonexistentToken { tokenId: token_id }.into())
            } else {
                Err(ERC721InsufficientApproval {
                    operator: spender,
                    tokenId: token_id,
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
    pub fn increase_balance(&mut self, account: Address, value: U256) {
        self.balances.setter(account).add_assign_unchecked(value)
    }

    /// Transfers `token_id` from its current owner to `to`, or alternatively mints (or burns) if the current owner
    /// (or `to`) is the zero address. Returns the owner of the `token_id` before the update.
    ///
    /// The `auth` argument is optional. If the value passed is non 0, then this function will check that
    /// `auth` is either the owner of the token, or approved to operate on the token (by the owner).
    ///
    /// Emits a {Transfer} event.
    ///
    /// NOTE: If overriding this function in a way that tracks balances, see also {_increaseBalance}.
    pub fn update(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Erc721Error> {
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

        evm::log(Transfer { from, to, tokenId: token_id });

        Ok(from)
    }

    /// Mints `token_id` and transfers it to `to`.
    ///
    /// WARNING: Usage of this method is discouraged, use {safe_mint} whenever possible
    ///
    /// Requirements:
    ///
    /// - `token_id` must not exist.
    /// - `to` cannot be the zero address.
    ///
    /// Emits a {Transfer} event.
    pub fn mint(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
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
    pub fn safe_mint(
        toplevel_storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Erc721Error> {
        toplevel_storage.borrow_mut().mint(to, token_id)?;
        Self::check_on_erc721_received(
            toplevel_storage,
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
    /// Requirements:
    ///
    /// - `token_id` must exist.
    ///
    /// Emits a {Transfer} event.
    pub fn burn(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        let previous_owner =
            self.update(Address::ZERO, token_id, Address::ZERO)?;
        if previous_owner == Address::ZERO {
            Err(ERC721NonexistentToken { tokenId: token_id }.into())
        } else {
            Ok(())
        }
    }

    /// Transfers `token_id` from `from` to `to`.
    ///  As opposed to {transferFrom}, this imposes no restrictions on msg.sender.
    ///
    /// Requirements:
    ///
    /// - `to` cannot be the zero address.
    /// - `token_id` token must be owned by `from`.
    ///
    /// Emits a {Transfer} event.
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        if to == Address::ZERO {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        let previous_owner = self.update(to, token_id, Address::ZERO)?;
        if previous_owner == Address::ZERO {
            Err(ERC721NonexistentToken { tokenId: token_id }.into())
        } else if previous_owner != from {
            Err(ERC721IncorrectOwner {
                sender: from,
                tokenId: token_id,
                owner: previous_owner,
            }
            .into())
        } else {
            Ok(())
        }
    }

    /// Same as {xref-ERC721-safe_transfer-address-address-uint256-}[`_safeTransfer`], with an additional `data` parameter which is
    /// forwarded in {IERC721Receiver-onERC721Received} to contract recipients.
    pub fn safe_transfer(
        storage: &mut (impl TopLevelStorage + BorrowMut<Self>),
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Erc721Error> {
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
    pub fn approve_inner(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
        emit_event: bool,
    ) -> Result<(), Erc721Error> {
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
                evm::log(Approval { owner, approved: to, tokenId: token_id })
            }
        }

        self.token_approvals.setter(token_id).set(to);
        Ok(())
    }

    /// Approve `operator` to operate on all of `owner` tokens
    ///
    /// Requirements:
    /// - operator can't be the address zero.
    ///
    /// Emits an {ApprovalForAll} event.
    pub fn set_approval_for_all_inner(
        &mut self,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), Erc721Error> {
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
    pub fn require_owned(
        &self,
        token_id: U256,
    ) -> Result<Address, Erc721Error> {
        let owner = self.owner_of_inner(token_id)?;
        if owner == Address::ZERO {
            return Err(ERC721NonexistentToken { tokenId: token_id }.into());
        }
        Ok(owner)
    }

    /// Performs an acceptance check for the provided `operator` by calling {IERC721-onERC721Received}
    /// on the `to` address. The `operator` is generally the address that initiated the token transfer (i.e. `msg.sender`).
    ///
    /// The acceptance call is not executed and treated as a no-op if the target address is doesn't contain code (i.e. an EOA).
    /// Otherwise, the recipient must implement {IERC721Receiver-onERC721Received} and return the acceptance magic value to accept
    /// the transfer.
    pub fn check_on_erc721_received(
        storage: &mut impl TopLevelStorage,
        operator: Address,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Erc721Error> {
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
                Err(err) => Err(Erc721Error(err.into())),
            };
        }
        Ok(())
    }
}

use stylus_sdk::storage::{StorageGuardMut, StorageUint};

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
