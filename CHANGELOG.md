# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Add `AccessControlEnumerable` extension that supports role member enumeration. #622
- Add `EnumerableAddressSet`. #622

### Changed

-

### Changed (Breaking)

- Remove implementation of `Deref<Target = Erc1155>` for `Erc1155Supply`, `Deref<Target = Erc721>` for `Erc721Consecutive`, and `Deref<Target = Ownable>` for `Ownable2Step`. #724

## [v0.2.0] - 2025-06-20

> **Heads-up:** this is the production release after four pre-releases
> (`alpha.1-4`, `rc.0`). The items below aggregate the _user-facing_ changes
> since **v0.1.2**. For the step-by-step evolution, see the individual
> pre-release sections further down.

### Added

- **ERC-1155 token** (`Erc1155`, `Burnable`, `MetadataUri`, `Supply`, `UriStorage`).
- **ERC-4626** “Tokenized Vault Standard” and **ERC-20 Flash Mint** extension.
- **ERC-2981** on-chain royalties.
- **ERC-20 Utils**: `SafeErc20`.
- **Finance**: `VestingWallet`.
- **Ownership**: `Ownable2Step`.
- **Token wrappers**: `Erc20Wrapper`, `Erc721Wrapper`.
- **Cryptography**: Poseidon2 hash, Pedersen hash (Starknet params), Short-Weierstrass Curves.
- **Math & utils**: optimised `Uint<_>` big-integer ops (`mul_div`, shift-left/right, checked/unchecked add/sub).
- **Constructors** now supported across all contracts.
- All events derive `Debug`; contracts implement `MethodError` and `IErc165`.

### Changed

- Keccak constants pre-computed at compile-time.
- Optimisations across contracts and crypto libraries.

### Changed (Breaking)

- **Stylus SDK** ↑ to **v0.9.0** (`cargo-stylus` ↑ to v0.6.0)
- Contracts refactored to new inheritance model.
- Interface IDs now returned by `fn interface_id()`; `IErc165::supports_interface` takes `&self`; `Erc165` struct removed.
- Public state fields made private.
- Feature **`std` removed** from libraries.

### Fixed

- Correct ERC-165 IDs for `IErc721Metadata`, `IErc721Enumerable`, etc.
- `#[interface_id]` macro now propagates super-traits correctly.
- Edge-cases in Merkle proofs, big-int overflows and re-entrancy bugs resolved.

## [v0.2.0-rc.0] - 2025-05-22

### Added

- Contracts now support constructors. #639
- Add Pedersen hash with Starknet parameters. #644
- Add shift left, right operators to Uint. #644
- `Erc721Wrapper` extension to support token wrapping. #461
- Add callable interface for ERC-721. #461
- Add missing functions to callable ERC-20 interface. #461
- All events now derive `Debug`. #614
- `Erc20Wrapper` extension to support token wrapping. #498
- `Erc20` events derive `Debug`. #498
- Implement `MethodError` for all contracts' errors. #594
- Implement `IErc165` for all base contracts for standard interface detection. #603
- Expose interface ID for `Erc20Wrapper`, `Erc4626` and `Erc20FlashMint`. #603
- Short Weierstrass elliptic curves primitives. #589

### Changed

- Optimize Stylus SDK imports. #598
- Updated the recommended way to inherit errors. #602

### Changed (Breaking)

- Bump Stylus SDK to `v0.9.0`. #639
- Convert associated `const INTERFACE_ID` into an associated `fn interface_id()` on all traits. #639
- `IErc165::supports_interface` now accepts `&self` as first parameter. #639
- Removed `Erc165` struct. #639
- Contracts now use the new Stylus SDK inheritance model. #639
- Moved `Erc20` callable interface to _/erc20/interface.rs_ module and renamed it to `Erc20Interface`. #461
- Bump `cargo-stylus` to `v0.5.11`. #617
- Bump Stylus SDK to `v0.8.4`. #624
- Remove `ownable_two_step::Error` wrapper in `Ownable2Step`, and emit `ownable::Error` directly. #594
- Poseidon babybear and goldilocks (64-bit) instances now have 256-bit security (capacity 4). #613
- `BitIteratorBE` (bit iteration) trait at `openzeppelin_crypto` now accepts `self` by value. #589
- Feature `std` was removed from libraries. #662

### Fixed

- The `#[interface_id]` attribute now correctly copies supertraits. #651
- `IErc721Metadata::interface_id()` now has the correct value.

## [v0.2.0-alpha.4] - 2025-03-06

### Added

- `Erc2981` contract. #508
- Implement `Deref<Target = Erc1155>` for `Erc1155Supply` and `Deref<Target = Erc721>` for `Erc721Consecutive`. #569
- Implement `Deref<Target = Ownable>` for `Ownable2Step`. #552

### Changed (Breaking)

- Refactor `Erc20Permit` extension to be a composition of `Erc20` and `Nonces` contracts. #574
- Replace `VestingWallet::receive_ether` with dedicated `receive` function. #529
- Extract `IAccessControl` trait from `AccessControl` contract. #527
- Bump Stylus SDK to `v0.8.1` #587

### Fixed

- `IErc165` implementations for `Erc721Metadata` and `Erc721Enumerable` now support ERC-165 interface ID. #570
- Handle missing leaves for non-trivial merkle trees. #578

## [v0.2.0-alpha.3] - 2025-01-30

### Added

- Optimised implementation of bigintegers `Uint<_>` for finite fields. #495
- `Erc4626` "Tokenized Vault Standard". #465
- Implement `mul_div` for `U256`. #465
- Implement `AddAssignChecked` for `StorageUint`. #474
- `Erc20FlashMint` extension. #407

### Changed

- Keccak constants `PERMIT_TYPEHASH` in `Erc20Permit`, and `TYPE_HASH` in `Erc712` are now statically computed. #478
- Use `AddAssignChecked` in `VestingWallet`, `Erc1155Supply`, `Erc1155`, `Erc20`, `Nonces`. #474
- Use `AddAssignUnchecked` and `SubAssignUnchecked` in `erc20::_update`. #467

### Changed (Breaking)

- All contract state fields are no longer public. #500
- `Erc721Consecutive::_mint_consecutive` turned into an internal function. #500
- Bump `cargo-stylus` to `v0.5.8`. #493
- Constants `TYPE_HASH`, `FIELDS`, `SALT` and `TYPED_DATA_PREFIX`, and type `DomainSeparatorTuple` are no longer exported from `utils::cryptography::eip712`. #478
- Bump Stylus SDK to v0.7.0. #433
- Bump `alloy` dependencies to v0.8.14. #433
- Add full support for reentrancy (changed `VestingWallet` signature for some functions). #407
- `Nonce::use_nonce` panics on exceeding `U256::MAX`. #467

## [v0.2.0-alpha.2] - 2024-12-18

### Added

- `Erc1155Supply` extension. #418
- `Erc1155Pausable`extension. #432
- `Erc1155UriStorage` extension. #431
- `VestingWallet` contract. #402
- `Erc1155Burnable` extension. #417
- `Erc1155MetadataUri` extension. #416
- `Poseidon2` sponge hash function. #388

### Changed

- Update "magic values" to explicit calculations in `Erc721Metadata::supports_interface`, and `Erc721::_check_on_erc721_received`. #442
- Implement `AddAssignUnchecked` and `SubAssignUnchecked` for `StorageUint`. #418
- Implement `MethodError` for `safe_erc20::Error`. #402
- Use `function_selector!` to calculate transfer type selector in `Erc1155`. #417

### Changed (Breaking)

- Update internal functions of `Erc721` and `Erc721Consecutive` to accept a reference to `Bytes`. #437

## [v0.2.0-alpha.1] - 2024-11-15

### Added

- ERC-1155 Multi Token Standard. #275
- `SafeErc20` Utility. #289
- Finite Fields arithmetic. #376
- `Ownable2Step` contract. #352
- `IOwnable` trait. #352

### Changed(breaking)

- Removed `only_owner` from the public interface of `Ownable`. #352

## [0.1.1] - 2024-10-28

### Changed

- Mini alloc is now used by default via the stylus-sdk. This avoids conflicts with duplicate `#[global_allocator]`
  definitions. #373
- Removed the panic handler from the library, making it easier for `std` and `no_std` projects to use the library. #373

## [0.1.0] - 2024-10-17

- Initial release
