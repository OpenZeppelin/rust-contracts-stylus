use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
    uint,
};
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    CacheOpt,
};

sol!(
    #[sol(rpc)]
    contract Erc4626 {
        function asset() public view  returns (address);
        function totalAssets() public view returns (uint256);
        function convertToShares(uint256 assets) public view  returns (uint256);
        function convertToAssets(uint256 shares) public view  returns (uint256);
        function maxMint(address receiver) public view  returns (uint256);
        function maxDeposit(address owner) public view  returns (uint256);
        function maxWithdraw(address owner) public view  returns (uint256);
        function maxRedeem(address owner) public view  returns (uint256);
        function previewDeposit(uint256 assets) public view  returns (uint256);
        function previewMint(uint256 shares) public view  returns (uint256);
        function previewRedeem(uint256 shares) public view  returns (uint256);
        function previewWithdraw(uint256 assets) public view  returns (uint256);
        function deposit(uint256 assets, address receiver) public  returns (uint256);
        function mint(uint256 shares, address receiver) public  returns (uint256);
        function redeem(uint256 shares, address receiver,address owner) public  returns (uint256);
        function withdraw(uint256 assets, address receiver,address owner) public  returns (uint256);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    let reports = run_with(CacheOpt::None).await?;
    let report = reports
        .into_iter()
        .try_fold(ContractReport::new("Erc4626"), ContractReport::add)?;

    let cached_reports = run_with(CacheOpt::Bid(0)).await?;
    let report = cached_reports
        .into_iter()
        .try_fold(report, ContractReport::add_cached)?;

    Ok(report)
}

pub async fn run_with(
    cache_opt: CacheOpt,
) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_addr = alice.address();
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let bob = Account::new().await?;
    let bob_addr = bob.address();

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc4626::new(contract_addr, &alice_wallet);

    // IMPORTANT: Order matters!
    use Erc4626::*;
    #[rustfmt::skip]
    let receipts = vec![
        (assetCall::SIGNATURE, receipt!(contract.asset())?),
        (totalAssetsCall::SIGNATURE, receipt!(contract.totalAssets())?),
        (convertToSharesCall::SIGNATURE, receipt!(contract.convertToShares(uint!(100_U256)))?),
        (convertToAssetsCall::SIGNATURE, receipt!(contract.convertToAssets(uint!(100_U256)))?),
        (maxMintCall::SIGNATURE, receipt!(contract.maxMint(bob_addr))?),
        (maxDepositCall::SIGNATURE, receipt!(contract.maxDeposit(alice_addr))?),
        (maxWithdrawCall::SIGNATURE, receipt!(contract.maxWithdraw(alice_addr))?),
        (maxRedeemCall::SIGNATURE, receipt!(contract.maxRedeem(alice_addr))?),
        (previewDepositCall::SIGNATURE, receipt!(contract.previewDeposit(uint!(100_U256)))?),
        (previewMintCall::SIGNATURE, receipt!(contract.previewMint(uint!(100_U256)))?),
        (previewRedeemCall::SIGNATURE, receipt!(contract.previewRedeem(uint!(100_U256)))?),
        (previewWithdrawCall::SIGNATURE, receipt!(contract.previewWithdraw(uint!(100_U256)))?),
        (depositCall::SIGNATURE, receipt!(contract.deposit(uint!(100_U256), bob_addr))?),
        (mintCall::SIGNATURE, receipt!(contract.mint(uint!(100_U256), bob_addr))?),
        (redeemCall::SIGNATURE, receipt!(contract.redeem(uint!(100_U256), bob_addr,alice_addr))?),
        (withdrawCall::SIGNATURE, receipt!(contract.withdraw(uint!(100_U256), bob_addr, alice_addr))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(
    account: &Account,
    cache_opt: CacheOpt,
) -> eyre::Result<Address> {
    crate::deploy(account, "Erc4626", None, cache_opt).await
}
