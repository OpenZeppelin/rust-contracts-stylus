// use std::error::Error;
//
// use alloy::{
//     primitives::{Address, FixedBytes, B256, U256},
//     providers::WalletProvider,
//     sol_types::SolError,
// };
// use eyre::Result;
//
// use alloy::sol;
//
// use crate::{context::build_context, error::Assert};
//
// sol!(
//     #[sol(rpc)]
//     contract Erc721 {
//         function name() external view returns (string memory);
//         function symbol() external view returns (string memory);
//         function tokenURI(uint256 token_id) external view returns (string memory);
//
//         function supportsInterface(bytes4 interface_id) external pure returns (bool);
//
//         function balanceOf(address owner) external view returns (uint256 balance);
//         function ownerOf(uint256 token_id) external view returns (address owner);
//         function safeTransferFrom(address from, address to, uint256 token_id) external;
//         function safeTransferFrom(address from, address to, uint256 token_id, bytes calldata data) external;
//         function transferFrom(address from, address to, uint256 token_id) external;
//         function approve(address to, uint256 token_id) external;
//         function setApprovalForAll(address operator, bool approved) external;
//         function getApproved(uint256 token_id) external view returns (address);
//         function isApprovedForAll(address owner, address operator) external view returns (bool);
//
//         function burn(uint256 token_id) external;
//         function mint(address to, uint256 token_id) external;
//
//         function paused() external view returns (bool);
//         function pause() external;
//         function unpause() external;
//
//         error ERC721InvalidOwner(address owner);
//         error ERC721NonexistentToken(uint256 tokenId);
//         error ERC721IncorrectOwner(address sender, uint256 tokenId, address owner);
//         error ERC721InvalidSender(address sender);
//         error ERC721InvalidReceiver(address receiver);
//         error ERC721InsufficientApproval(address operator, uint256 tokenId);
//         error ERC721InvalidApprover(address approver);
//         error ERC721InvalidOperator(address operator);
//
//         error EnforcedPause();
//         error ExpectedPause();
//     }
// );
//
// fn random_uint() -> U256 {
//     FixedBytes::<32>::random().into()
// }
//
// #[tokio::test]
// async fn mint() -> Result<()> {
//     let ctx = build_context();
//     let alice = &ctx.signers()[0];
//     let alice_addr = alice.default_signer_address();
//     let contract = Erc721::new(Address::random(), &alice);
//
//     let token_id = random_uint();
//
//     let _ = contract.mint(alice_addr, token_id).send().await?;
//     let Erc721::ownerOfReturn { owner } =
//         contract.ownerOf(token_id).call().await?;
//     assert_eq!(owner, alice_addr);
//
//     let Erc721::balanceOfReturn { balance } =
//         contract.balanceOf(alice_addr).call().await?;
//     assert!(balance >= U256::from(1));
//     Ok(())
// }
//
// #[tokio::test]
// async fn error_when_reusing_token_id() -> Result<()> {
//     let ctx = build_context();
//     let alice = &ctx.signers()[0];
//     let alice_addr = alice.default_signer_address();
//     let contract = Erc721::new(Address::random(), &alice);
//
//     let token_id = random_uint();
//     let _ = contract.mint(alice_addr, token_id).send().await;
//     let err = contract
//         .mint(alice_addr, token_id)
//         .send()
//         .await
//         .expect_err("should not mint a token id twice");
//
//     err.assert(Erc721::ERC721InvalidSender { sender: Address::ZERO });
//     Ok(())
// }

// #[tokio::test]
// async fn transfer() -> Result<()> {
//     let ctx = build_context();
//     let alice = &ctx.signers()[0];
//     let alice_addr = alice.default_signer_address();
//     let contract = Erc721::new(Address::random(), &alice);
//
//     let token_id = random_uint();
//     let _ = contract.mint(alice_addr, token_id).send().await?;
//     let _ = contract
//         .transferFrom(alice_addr, bob.wallet.address(), token_id)
//         .send()
//         .await?;
//     let owner = bob.ownerOf(token_id).call().await?;
//     assert_eq!(owner, bob.wallet.address());
//     Ok(())
// }
//
// #[tokio::test]
// async fn error_when_transfer_nonexistent_token() -> Result<()> {
//     let ctx = build_context();
//     let alice = &ctx.signers()[0];
//     let alice_addr = alice.default_signer_address();
//     let contract = Erc721::new(Address::random(), &alice);
//
//     let token_id = random_uint();
//     let err = contract
//         .transferFrom(alice_addr, bob.wallet.address(), token_id)
//         .send()
//         .await
//         .expect_err("should not transfer a non existent token");
//     err.assert(ERC721NonexistentToken { token_id })
// }
//
// #[tokio::test]
// async fn approve_token_transfer() -> Result<()> {
//     let ctx = build_context();
//     let alice = &ctx.signers()[0];
//     let alice_addr = alice.default_signer_address();
//     let contract = Erc721::new(Address::random(), &alice);
//
//     let token_id = random_uint();
//     let _ = contract.mint(alice_addr, token_id).send().await?;
//     let _ = contract.approve(bob.wallet.address(), token_id).send().await?;
//     let _ = bob
//         .transferFrom(alice_addr, bob.wallet.address(), token_id)
//         .send()
//         .await?;
//     let owner = bob.ownerOf(token_id).call().await?;
//     assert_eq!(owner, bob.wallet.address());
//     Ok(())
// }
//
// #[tokio::test]
// async fn error_when_transfer_unapproved_token() -> Result<()> {
//     let ctx = build_context();
//     let alice = &ctx.signers()[0];
//     let alice_addr = alice.default_signer_address();
//     let contract = Erc721::new(Address::random(), &alice);
//
//     let token_id = random_uint();
//     let _ = contract.mint(alice_addr, token_id).send().await?;
//     let err = bob
//         .transferFrom(alice_addr, bob.wallet.address(), token_id)
//         .send()
//         .await
//         .expect_err("should not transfer unapproved token");
//     err.assert(ERC721InsufficientApproval {
//         operator: bob.wallet.address(),
//         token_id,
//     })
// }
//
// // TODO: add more tests for erc721
