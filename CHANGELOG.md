# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Update internal functions of `Erc721` and `Erc721Consecutive` accept reference to `Bytes`. #437

### Fixed

-

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
