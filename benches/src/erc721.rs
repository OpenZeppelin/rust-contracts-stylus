use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolConstructor,
    uint,
};
use e2e::{receipt, Account};

use crate::ArbOtherFields;

sol!(
    #[sol(rpc)]
    contract Erc721 {
        function balanceOf(address owner) external view returns (uint256 balance);
        function approve(address to, uint256 tokenId) external;
        function getApproved(uint256 tokenId) external view returns (address approved);
        function isApprovedForAll(address owner, address operator) external view returns (bool approved);
        function ownerOf(uint256 tokenId) external view returns (address ownerOf);
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function setApprovalForAll(address operator, bool approved) external;
        function totalSupply() external view returns (uint256 totalSupply);
        function transferFrom(address from, address to, uint256 tokenId) external;
        function mint(address to, uint256 tokenId) external;
        function burn(uint256 tokenId) external;
    }
);

sol!("../examples/erc721/src/constructor.sol");

pub async fn bench() -> eyre::Result<()> {
    let alice = Account::new().await?;
    let alice_addr = alice.address();
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let bob = Account::new().await?;
    let bob_addr = bob.address();

    let contract_addr = deploy(&alice).await;
    let contract = Erc721::new(contract_addr, &alice_wallet);

    let _ = receipt!(contract.mint(alice_addr, uint!(2_U256)))?;
    let _ = receipt!(contract.mint(alice_addr, uint!(3_U256)))?;
    let _ = receipt!(contract.mint(alice_addr, uint!(4_U256)))?;

    // IMPORTANT: Order matters!
    let receipts = vec![
        ("balanceOf(address owner) external view returns (uint256 balance)", receipt!(contract.balanceOf(alice_addr))?),
        ("approve(address to, uint256 tokenId)", receipt!(contract.approve(bob_addr, uint!(2_U256)))?),
        ("getApproved(uint256 tokenId) external view returns (address approved)", receipt!(contract.getApproved(uint!(2_U256)))?),
        ("isApprovedForAll(address owner, address operator) external view returns (bool approved)", receipt!(contract.isApprovedForAll(alice_addr, bob_addr))?),
        ("ownerOf(uint256 tokenId) external view returns (address ownerOf)", receipt!(contract.ownerOf(uint!(2_U256)))?),
        ("safeTransferFrom(address from, address to, uint256 tokenId)", receipt!(contract.safeTransferFrom(alice_addr, bob_addr, uint!(3_U256)))?),
        ("setApprovalForAll(address operator, bool approved)", receipt!(contract.setApprovalForAll(bob_addr, true))?),
        ("totalSupply() external view returns (uint256 totalSupply)", receipt!(contract.totalSupply())?),
        ("transferFrom(address from, address to, uint256 tokenId)", receipt!(contract.transferFrom(alice_addr, bob_addr, uint!(4_U256)))?),
        ("mint(address to, uint256 tokenId)", receipt!(contract.mint(alice_addr, uint!(1_U256)))?),
        ("burn(uint256 tokenId)", receipt!(contract.burn(uint!(1_U256)))?),

    ];

    // Calculate the width of the longest function name.
    let max_name_width = receipts
        .iter()
        .max_by_key(|x| x.0.len())
        .expect("should at least bench one function")
        .0
        .len();
    let name_width = max_name_width.max("ERC-20".len());

    // Calculate the total width of the table.
    let total_width = name_width + 3 + 6 + 3 + 6 + 3 + 20 + 4; // 3 for padding, 4 for outer borders

    // Print the table header.
    println!("+{}+", "-".repeat(total_width - 2));
    println!(
        "| {:<width$} | L2 Gas | L1 Gas |        Effective Gas |",
        "ERC-20",
        width = name_width
    );
    println!(
        "|{}+--------+--------+----------------------|",
        "-".repeat(name_width + 2)
    );

    // Print each row.
    for (func_name, receipt) in receipts {
        let l2_gas = receipt.gas_used;
        let arb_fields: ArbOtherFields = receipt.other.deserialize_into()?;
        let l1_gas = arb_fields.gas_used_for_l1.to::<u128>();
        let effective_gas = l2_gas - l1_gas;

        println!(
            "| {:<width$} | {:>6} | {:>6} | {:>20} |",
            func_name,
            l2_gas,
            l1_gas,
            effective_gas,
            width = name_width
        );
    }

    // Print the table footer.
    println!("+{}+", "-".repeat(total_width - 2));

    Ok(())
}

async fn deploy(account: &Account) -> Address {
    let args = Erc721Example::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "erc721", Some(args)).await
}
