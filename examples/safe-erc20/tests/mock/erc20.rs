#![allow(dead_code)]
#![cfg(feature = "e2e")]
use alloy::{primitives::Address, sol};
use e2e::Wallet;

sol! {
    #[allow(missing_docs)]
    // Built with Remix IDE; solc v0.8.21+commit.d9974bed
    #[sol(rpc, bytecode="608060405234801561000f575f80fd5b506104278061001d5f395ff3fe608060405234801561000f575f80fd5b506004361061003f575f3560e01c806340c10f191461004357806370a082311461005f578063a9059cbb1461008f575b5f80fd5b61005d6004803603810190610058919061029a565b6100bf565b005b610079600480360381019061007491906102d8565b610115565b6040516100869190610312565b60405180910390f35b6100a960048036038101906100a4919061029a565b61015a565b6040516100b69190610345565b60405180910390f35b805f808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825461010a919061038b565b925050819055505050565b5f805f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050919050565b5f815f803373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546101a691906103be565b92505081905550815f808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8282546101f8919061038b565b925050819055506001905092915050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6102368261020d565b9050919050565b6102468161022c565b8114610250575f80fd5b50565b5f813590506102618161023d565b92915050565b5f819050919050565b61027981610267565b8114610283575f80fd5b50565b5f8135905061029481610270565b92915050565b5f80604083850312156102b0576102af610209565b5b5f6102bd85828601610253565b92505060206102ce85828601610286565b9150509250929050565b5f602082840312156102ed576102ec610209565b5b5f6102fa84828501610253565b91505092915050565b61030c81610267565b82525050565b5f6020820190506103255f830184610303565b92915050565b5f8115159050919050565b61033f8161032b565b82525050565b5f6020820190506103585f830184610336565b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f61039582610267565b91506103a083610267565b92508282019050808211156103b8576103b761035e565b5b92915050565b5f6103c882610267565b91506103d383610267565b92508282039050818111156103eb576103ea61035e565b5b9291505056fea264697066735822122057c469102c1fccf2c882c0a431f0687b30ef2c58ab13e2ecfc052290af551aa264736f6c63430008150033")]
    contract ERC20Mock {
        mapping(address => uint256) private _balances;

        function balanceOf(address account) public view returns (uint256) {
            return _balances[account];
        }

        function mint(address account, uint256 amount) public {
            _balances[account] += amount;
        }

        function transfer(address to, uint256 amount) public returns (bool) {
            _balances[msg.sender] -= amount;
            _balances[to] += amount;
            return true;
        }
    }
}

pub async fn deploy(wallet: &Wallet) -> eyre::Result<Address> {
    // Deploy the contract.
    let contract = ERC20Mock::deploy(wallet).await?;
    Ok(*contract.address())
}
