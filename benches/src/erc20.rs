use alloy::{primitives::Address, sol, sol_types::SolConstructor, uint};
use alloy_primitives::U256;
use e2e::Account;

sol!(
    #[sol(rpc)]
    contract Erc20 {
        function name() external view returns (string name);
        function symbol() external view returns (string symbol);
        function decimals() external view returns (uint8 decimals);
        function totalSupply() external view returns (uint256 totalSupply);
        function balanceOf(address account) external view returns (uint256 balance);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256 allowance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function cap() public view virtual returns (uint256 cap);

        function mint(address account, uint256 amount) external;
        function burn(uint256 amount) external;
        function burnFrom(address account, uint256 amount) external;

        error ERC20ExceededCap(uint256 increased_supply, uint256 cap);
        error ERC20InvalidCap(uint256 cap);

        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        error ERC20InvalidSender(address sender);
        error ERC20InvalidReceiver(address receiver);
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        error ERC20InvalidSpender(address spender);

        #[derive(Debug, PartialEq)]
        event Transfer(address indexed from, address indexed to, uint256 value);

        #[derive(Debug, PartialEq)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
);

sol!("../examples/erc20/src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";
const CAP: U256 = uint!(1_000_000_U256);

async fn deploy(
    rpc_url: &str,
    private_key: &str,
    cap: Option<U256>,
) -> eyre::Result<Address> {
    let args = Erc20Example::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        cap_: cap.unwrap_or(CAP),
    };
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await
}

pub async fn bench() -> eyre::Result<()> {
    let alice = Account::new().await?;
    let contract_addr = deploy(alice.url(), &alice.pk(), None).await?;
    let contract = Erc20::new(contract_addr, &alice.wallet);

    let gas = contract.name().estimate_gas().await?;
    println!("name(): {gas}");
    let gas = contract.symbol().estimate_gas().await?;
    println!("symbol(): {gas}");
    let gas = contract.cap().estimate_gas().await?;
    println!("cap(): {gas}");
    let gas = contract.decimals().estimate_gas().await?;
    println!("decimals(): {gas}");
    let gas = contract.totalSupply().estimate_gas().await?;
    println!("totalSupply(): {gas}");

    Ok(())
}
