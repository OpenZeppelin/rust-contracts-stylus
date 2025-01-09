use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::{SolCall, SolConstructor},
    uint,
};
use alloy_primitives::U256;
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
    contract VestingWallet {
        function owner() public view virtual returns (address owner);
        function receiveEther() external payable virtual;
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

sol!("../examples/vesting-wallet/src/constructor.sol");
sol!("../examples/erc20/src/constructor.sol");

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

    let _ = receipt!(contract.receiveEther().value(uint!(1000_U256)))?;
    let _ = receipt!(erc20.mint(contract_addr, uint!(1000_U256)))?;

    // IMPORTANT: Order matters!
    use VestingWallet::*;
    #[rustfmt::skip]
    let receipts = vec![
        (receiveEtherCall::SIGNATURE, receipt!(contract.receiveEther())?),
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
    let args = VestingWalletExample::constructorCall {
        beneficiary: account.address(),
        startTimestamp: START_TIMESTAMP,
        durationSeconds: DURATION_SECONDS,
    };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "vesting-wallet", Some(args), cache_opt).await
}

async fn deploy_token(
    account: &Account,
    cache_opt: Opt,
) -> eyre::Result<Address> {
    let args = Erc20Example::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        cap_: CAP,
    };
    let args = alloy::hex::encode(args.abi_encode());
    crate::deploy(account, "erc20", Some(args), cache_opt).await
}
