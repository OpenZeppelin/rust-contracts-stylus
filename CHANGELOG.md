# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.0-alpha.1] - 2024-11-15

### Added

- ERC-1155 Multi Token Standard. #275
- `SafeErc20` Utility. #289
- Finite Fields arithmetics. #376
- `Ownable2Step` contract. #352
- `IOwnable` trait. #352
- `Poseidon2` sponge hash function. #388

### Changed(breaking)

- Removed `only_owner` from the public interface of `Ownable`. #352

### Changed

-

### Fixed

-

## [0.1.1] - 2024-10-28

### Changed

- Mini alloc is now used by default via the stylus-sdk. This avoids conflicts with duplicate `#[global_allocator]`
  definitions. #373
- Removed the panic handler from the library, making it easier for `std` and `no_std` projects to use the library. #373

## [0.1.0] - 2024-10-17

- Initial release
