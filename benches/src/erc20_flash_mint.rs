use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::{SolCall, SolConstructor},
    uint,
};
use alloy_primitives::{address, U256};
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    CacheOpt,
};

sol!(
    #[sol(rpc)]
    contract Erc20FlashMint {
        function maxFlashLoan(address token) external view returns (uint256 maxLoan);
        function flashFee(address token, uint256 amount) external view returns (uint256 fee);
        function flashLoan(address receiver, address token, uint256 amount, bytes calldata data) external returns (bool);
    }
);

sol!("../examples/erc20-flash-mint/src/constructor.sol");
sol!("../examples/erc20-flash-mint/src/ERC3156FlashBorrowerMock.sol");

const FEE_RECEIVER: Address =
    address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
const FLASH_FEE_AMOUNT: U256 = uint!(100_U256);

pub async fn bench() -> eyre::Result<ContractReport> {
    let reports = run_with(CacheOpt::None).await?;
    let report = reports
        .into_iter()
        .try_fold(ContractReport::new("Erc20FlashMint"), ContractReport::add)?;

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
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt.clone()).await?;

    let receiver_addr = deploy_receiver(&alice, cache_opt).await?;

    let contract = Erc20FlashMint::new(contract_addr, &alice_wallet);

    let amount = uint!(100_U256);

    // IMPORTANT: Order matters!
    use Erc20FlashMint::*;
    #[rustfmt::skip]
    let receipts = vec![
        (maxFlashLoanCall::SIGNATURE, receipt!(contract.maxFlashLoan(contract_addr))?),
        (flashFeeCall::SIGNATURE, receipt!(contract.flashFee(contract_addr, amount))?),
        (flashLoanCall::SIGNATURE, receipt!(contract.flashLoan(receiver_addr, contract_addr, amount, vec![].into()))?),
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
    let args = Erc20FlashMintExample::constructorCall {
        flashFeeAmount_: FLASH_FEE_AMOUNT,
        flashFeeReceiverAddress_: FEE_RECEIVER,
    };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "erc20-flash-mint", Some(args), cache_opt).await
}

async fn deploy_receiver(
    account: &Account,
    cache_opt: CacheOpt,
) -> eyre::Result<Address> {
    let args = ERC3156FlashBorrowerMock::constructorCall {
        enableApprove: true,
        enableReturn: true,
    };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "erc3156-flash-borrower-mock", Some(args), cache_opt)
        .await
}
