use alloy::{
    network::{AnyNetwork, EthereumWallet, TransactionBuilder},
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::{serde_helpers::WithOtherFields, TransactionRequest},
    sol,
    sol_types::SolCall,
    uint,
};
use alloy_primitives::U256;
use e2e::{constructor, receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
    contract VestingWallet {
        function owner() public view virtual returns (address owner);
        function start() external view returns (uint256 start);
        function duration() external view returns (uint256 duration);
        function end() external view returns (uint256 end);
        function released() external view returns (uint256 released);
        function released(address token) external view returns (uint256 released);
        function releasable() external view returns (uint256 releasable);
        function releasable(address token) external view returns (uint256 releasable);
        function release() external;
        function release(address token) external;
        function vestedAmount(uint64 timestamp) external view returns (uint256 vestedAmount);
        function vestedAmount(address token, uint64 timestamp) external view returns (uint256 vestedAmount);
    }

    #[sol(rpc)]
    contract Erc20 {
        function mint(address account, uint256 amount) external;
    }
);

const START_TIMESTAMP: u64 = 1000;
const DURATION_SECONDS: u64 = 1000;

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";
const CAP: U256 = uint!(1_000_000_U256);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("VestingWallet", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt.clone()).await?;
    let erc20_addr = deploy_token(&alice, cache_opt).await?;

    let contract = VestingWallet::new(contract_addr, &alice_wallet);
    let erc20 = Erc20::new(erc20_addr, &alice_wallet);

    let tx: WithOtherFields<TransactionRequest> = WithOtherFields {
        inner: TransactionRequest::default()
            .with_from(alice.address())
            .with_to(contract_addr)
            .with_value(uint!(1000_U256)),
        other: Default::default(),
    };

    alice_wallet.send_transaction(tx).await?.watch().await?;
    receipt!(erc20.mint(contract_addr, uint!(1000_U256)))?;

    // IMPORTANT: Order matters!
    use VestingWallet::*;
    #[rustfmt::skip]
    let receipts = vec![
        (startCall::SIGNATURE, receipt!(contract.start())?),
        (durationCall::SIGNATURE, receipt!(contract.duration())?),
        (endCall::SIGNATURE, receipt!(contract.end())?),
        (released_0Call::SIGNATURE, receipt!(contract.released_0())?),
        (released_1Call::SIGNATURE, receipt!(contract.released_1(erc20_addr))?),
        (releasable_0Call::SIGNATURE, receipt!(contract.releasable_0())?),
        (releasable_1Call::SIGNATURE, receipt!(contract.releasable_1(erc20_addr))?),
        (release_0Call::SIGNATURE, receipt!(contract.release_0())?),
        (release_1Call::SIGNATURE, receipt!(contract.release_1(erc20_addr))?),
        (vestedAmount_0Call::SIGNATURE, receipt!(contract.vestedAmount_0(START_TIMESTAMP))?),
        (vestedAmount_1Call::SIGNATURE, receipt!(contract.vestedAmount_1(erc20_addr, START_TIMESTAMP))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(
        account,
        "vesting-wallet",
        Some(constructor!(
            account.address(),
            START_TIMESTAMP,
            DURATION_SECONDS
        )),
        cache_opt,
    )
    .await
}

async fn deploy_token(
    account: &Account,
    cache_opt: Opt,
) -> eyre::Result<Address> {
    crate::deploy(
        account,
        "erc20",
        Some(constructor!(
            TOKEN_NAME.to_string(),
            TOKEN_SYMBOL.to_string(),
            CAP
        )),
        cache_opt,
    )
    .await
}
