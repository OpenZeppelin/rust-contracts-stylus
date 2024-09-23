#![allow(dead_code)]
#![cfg(feature = "e2e")]
use alloy::{primitives::Address, sol};
use e2e::Wallet;

sol! {
    #[allow(missing_docs)]
    // Built with Remix IDE; solc v0.8.21+commit.d9974bed
    #[sol(rpc, bytecode="608060405234801562000010575f80fd5b506040518060400160405280600781526020017f4d79546f6b656e000000000000000000000000000000000000000000000000008152506040518060400160405280600381526020017f4d544b000000000000000000000000000000000000000000000000000000000081525081600390816200008e91906200030d565b508060049081620000a091906200030d565b505050620003f1565b5f81519050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f60028204905060018216806200012557607f821691505b6020821081036200013b576200013a620000e0565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f600883026200019f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8262000162565b620001ab868362000162565b95508019841693508086168417925050509392505050565b5f819050919050565b5f819050919050565b5f620001f5620001ef620001e984620001c3565b620001cc565b620001c3565b9050919050565b5f819050919050565b6200021083620001d5565b620002286200021f82620001fc565b8484546200016e565b825550505050565b5f90565b6200023e62000230565b6200024b81848462000205565b505050565b5b818110156200027257620002665f8262000234565b60018101905062000251565b5050565b601f821115620002c1576200028b8162000141565b620002968462000153565b81016020851015620002a6578190505b620002be620002b58562000153565b83018262000250565b50505b505050565b5f82821c905092915050565b5f620002e35f1984600802620002c6565b1980831691505092915050565b5f620002fd8383620002d2565b9150826002028217905092915050565b6200031882620000a9565b67ffffffffffffffff811115620003345762000333620000b3565b5b6200034082546200010d565b6200034d82828562000276565b5f60209050601f83116001811462000383575f84156200036e578287015190505b6200037a8582620002f0565b865550620003e9565b601f198416620003938662000141565b5f5b82811015620003bc5784890151825560018201915060208501945060208101905062000395565b86831015620003dc5784890151620003d8601f891682620002d2565b8355505b6001600288020188555050505b505050505050565b610ed380620003ff5f395ff3fe608060405234801561000f575f80fd5b506004361061009c575f3560e01c806340c10f191161006457806340c10f191461015a57806370a082311461017657806395d89b41146101a6578063a9059cbb146101c4578063dd62ed3e146101f45761009c565b806306fdde03146100a0578063095ea7b3146100be57806318160ddd146100ee57806323b872dd1461010c578063313ce5671461013c575b5f80fd5b6100a8610224565b6040516100b59190610b4c565b60405180910390f35b6100d860048036038101906100d39190610bfd565b6102b4565b6040516100e59190610c55565b60405180910390f35b6100f66102d6565b6040516101039190610c7d565b60405180910390f35b61012660048036038101906101219190610c96565b6102df565b6040516101339190610c55565b60405180910390f35b61014461030d565b6040516101519190610d01565b60405180910390f35b610174600480360381019061016f9190610bfd565b610315565b005b610190600480360381019061018b9190610d1a565b610323565b60405161019d9190610c7d565b60405180910390f35b6101ae610334565b6040516101bb9190610b4c565b60405180910390f35b6101de60048036038101906101d99190610bfd565b6103c4565b6040516101eb9190610c55565b60405180910390f35b61020e60048036038101906102099190610d45565b6103d7565b60405161021b9190610c7d565b60405180910390f35b60606003805461023390610db0565b80601f016020809104026020016040519081016040528092919081815260200182805461025f90610db0565b80156102aa5780601f10610281576101008083540402835291602001916102aa565b820191905f5260205f20905b81548152906001019060200180831161028d57829003601f168201915b5050505050905090565b5f806102be610459565b90506102cb818585610460565b600191505092915050565b5f600254905090565b5f806102e9610459565b90506102f6858285610472565b610301858585610504565b60019150509392505050565b5f6012905090565b61031f82826105f4565b5050565b5f61032d82610673565b9050919050565b60606004805461034390610db0565b80601f016020809104026020016040519081016040528092919081815260200182805461036f90610db0565b80156103ba5780601f10610391576101008083540402835291602001916103ba565b820191905f5260205f20905b81548152906001019060200180831161039d57829003601f168201915b5050505050905090565b5f6103cf83836106b8565b905092915050565b5f60015f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054905092915050565b5f33905090565b61046d83838360016106da565b505050565b5f61047d84846103d7565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146104fe57818110156104ef578281836040517ffb8f41b20000000000000000000000000000000000000000000000000000000081526004016104e693929190610def565b60405180910390fd5b6104fd84848484035f6106da565b5b50505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610574575f6040517f96c6fd1e00000000000000000000000000000000000000000000000000000000815260040161056b9190610e24565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16036105e4575f6040517fec442f050000000000000000000000000000000000000000000000000000000081526004016105db9190610e24565b60405180910390fd5b6105ef8383836108a9565b505050565b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610664575f6040517fec442f0500000000000000000000000000000000000000000000000000000000815260040161065b9190610e24565b60405180910390fd5b61066f5f83836108a9565b5050565b5f805f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050919050565b5f806106c2610459565b90506106cf818585610504565b600191505092915050565b5f73ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff160361074a575f6040517fe602df050000000000000000000000000000000000000000000000000000000081526004016107419190610e24565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036107ba575f6040517f94280d620000000000000000000000000000000000000000000000000000000081526004016107b19190610e24565b60405180910390fd5b8160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f208190555080156108a3578273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161089a9190610c7d565b60405180910390a35b50505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036108f9578060025f8282546108ed9190610e6a565b925050819055506109c7565b5f805f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054905081811015610982578381836040517fe450d38c00000000000000000000000000000000000000000000000000000000815260040161097993929190610def565b60405180910390fd5b8181035f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550505b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610a0e578060025f8282540392505081905550610a58565b805f808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef83604051610ab59190610c7d565b60405180910390a3505050565b5f81519050919050565b5f82825260208201905092915050565b5f5b83811015610af9578082015181840152602081019050610ade565b5f8484015250505050565b5f601f19601f8301169050919050565b5f610b1e82610ac2565b610b288185610acc565b9350610b38818560208601610adc565b610b4181610b04565b840191505092915050565b5f6020820190508181035f830152610b648184610b14565b905092915050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610b9982610b70565b9050919050565b610ba981610b8f565b8114610bb3575f80fd5b50565b5f81359050610bc481610ba0565b92915050565b5f819050919050565b610bdc81610bca565b8114610be6575f80fd5b50565b5f81359050610bf781610bd3565b92915050565b5f8060408385031215610c1357610c12610b6c565b5b5f610c2085828601610bb6565b9250506020610c3185828601610be9565b9150509250929050565b5f8115159050919050565b610c4f81610c3b565b82525050565b5f602082019050610c685f830184610c46565b92915050565b610c7781610bca565b82525050565b5f602082019050610c905f830184610c6e565b92915050565b5f805f60608486031215610cad57610cac610b6c565b5b5f610cba86828701610bb6565b9350506020610ccb86828701610bb6565b9250506040610cdc86828701610be9565b9150509250925092565b5f60ff82169050919050565b610cfb81610ce6565b82525050565b5f602082019050610d145f830184610cf2565b92915050565b5f60208284031215610d2f57610d2e610b6c565b5b5f610d3c84828501610bb6565b91505092915050565b5f8060408385031215610d5b57610d5a610b6c565b5b5f610d6885828601610bb6565b9250506020610d7985828601610bb6565b9150509250929050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f6002820490506001821680610dc757607f821691505b602082108103610dda57610dd9610d83565b5b50919050565b610de981610b8f565b82525050565b5f606082019050610e025f830186610de0565b610e0f6020830185610c6e565b610e1c6040830184610c6e565b949350505050565b5f602082019050610e375f830184610de0565b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f610e7482610bca565b9150610e7f83610bca565b9250828201905080821115610e9757610e96610e3d565b5b9291505056fea2646970667358221220ce6ad8a217435fe22e4de3a933dfc9f416d3df7a68c81a7e6083d9cfbcae807c64736f6c63430008150033")]
    // SPDX-License-Identifier: MIT
    contract ERC20Mock is ERC20 {
        constructor() ERC20("MyToken", "MTK") {}

        function balanceOf(address account) public override view returns (uint256) {
            return super.balanceOf(account);
        }

        function mint(address account, uint256 value) public {
            super._mint(account, value);
        }

        function transfer(address to, uint256 amount) public override returns (bool) {
            return super.transfer(to, amount);
        }
    }
}

pub async fn deploy(wallet: &Wallet) -> eyre::Result<Address> {
    // Deploy the contract.
    let contract = ERC20Mock::deploy(wallet).await?;
    Ok(*contract.address())
}
