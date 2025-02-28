# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-

### Changed

- Implement `Deref<Target = Erc1155>` for `Erc1155Supply` and `Deref<Target = Erc721>` for `Erc721Consecutive`. #569
- Implement `Deref<Target = Ownable>` for `Ownable2Step` and `Deref<Target = Erc20>` for `Erc20Permit`. #552

### Changed (Breaking)

- Replace `VestingWallet::receive_ether` with dedicated `receive` function. #529
- Extract `IAccessControl` trait from `AccessControl` contract. #527

### Fixed

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
- Bump cargo-stylus to v0.5.8. #493
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
