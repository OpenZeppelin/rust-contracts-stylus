# E2E Grip - End-to-end Testing for Stylus

This end-to-end testing crate allows to create users, deploy contracts and
test all necessary scenarios you will probably need. This crate coupled with
[nitro test node](https://github.com/OffchainLabs/nitro-testnode) developed
by Offchain Labs and requires it to be installed to perform integration testing.

## Usage

Abi declaration:
```rust
use e2e_grip::prelude::*;

abigen!(
    Erc721Token,
    r#"[
        function ownerOf(uint256 token_id) external view returns (address)
        function transferFrom(address from, address to, uint256 token_id) external
        function mint(address to, uint256 token_id) external

        error ERC721InvalidOwner(address owner)
        error ERC721NonexistentToken(uint256 tokenId)
        error ERC721IncorrectOwner(address sender, uint256 tokenId, address owner)
    ]"#
);

pub type Erc721 = Erc721Token<HttpMiddleware>;
link_to_crate!(Erc721, "erc721-example");
```

Test case example:
```rust
#[e2e_grip::test]
async fn transfer(alice: User, bob: User) -> Result<()> {
    let erc721 = &alice.deploys::<Erc721>().await?;
    let token_id = random_token_id();
    let _ =
        alice.uses(erc721).mint(alice.address(), token_id).ctx_send().await?;
    let _ = alice
        .uses(erc721)
        .transfer_from(alice.address(), bob.address(), token_id)
        .ctx_send()
        .await?;
    let owner = bob.uses(erc721).owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, bob.address());
    Ok(())
}
```

### Notice

[code of conduct]: ../../CODE_OF_CONDUCT.md

[contribution guidelines]: ../../CONTRIBUTING.md

## Security

> [!WARNING]
> This project is still in a very early and experimental phase. It has never
> been audited nor thoroughly reviewed for security vulnerabilities. Do not use
> in production.

Refer to our [Security Policy](../../SECURITY.md) for more details.
