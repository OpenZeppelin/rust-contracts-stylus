#![allow(dead_code)]
#![cfg(feature = "e2e")]
use alloy::{primitives::Address, sol};
use e2e::Wallet;

sol! {
    #[allow(missing_docs)]
    // Built with Remix IDE; solc v0.8.21+commit.d9974bed
    #[sol(rpc, bytecode="608060405234801562000010575f80fd5b506040518060400160405280601181526020017f45524332304e6f52657475726e4d6f636b0000000000000000000000000000008152506040518060400160405280600381526020017f4e524d000000000000000000000000000000000000000000000000000000000081525081600390816200008e91906200030d565b508060049081620000a091906200030d565b505050620003f1565b5f81519050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f60028204905060018216806200012557607f821691505b6020821081036200013b576200013a620000e0565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f600883026200019f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8262000162565b620001ab868362000162565b95508019841693508086168417925050509392505050565b5f819050919050565b5f819050919050565b5f620001f5620001ef620001e984620001c3565b620001cc565b620001c3565b9050919050565b5f819050919050565b6200021083620001d5565b620002286200021f82620001fc565b8484546200016e565b825550505050565b5f90565b6200023e62000230565b6200024b81848462000205565b505050565b5b818110156200027257620002665f8262000234565b60018101905062000251565b5050565b601f821115620002c1576200028b8162000141565b620002968462000153565b81016020851015620002a6578190505b620002be620002b58562000153565b83018262000250565b50505b505050565b5f82821c905092915050565b5f620002e35f1984600802620002c6565b1980831691505092915050565b5f620002fd8383620002d2565b9150826002028217905092915050565b6200031882620000a9565b67ffffffffffffffff811115620003345762000333620000b3565b5b6200034082546200010d565b6200034d82828562000276565b5f60209050601f83116001811462000383575f84156200036e578287015190505b6200037a8582620002f0565b865550620003e9565b601f198416620003938662000141565b5f5b82811015620003bc5784890151825560018201915060208501945060208101905062000395565b86831015620003dc5784890151620003d8601f891682620002d2565b8355505b6001600288020188555050505b505050505050565b610f0180620003ff5f395ff3fe608060405234801561000f575f80fd5b506004361061009c575f3560e01c806340c10f191161006457806340c10f191461015a57806370a082311461017657806395d89b41146101a6578063a9059cbb146101c4578063dd62ed3e146101f45761009c565b806306fdde03146100a0578063095ea7b3146100be57806318160ddd146100ee57806323b872dd1461010c578063313ce5671461013c575b5f80fd5b6100a8610224565b6040516100b59190610b7a565b60405180910390f35b6100d860048036038101906100d39190610c2b565b6102b4565b6040516100e59190610c83565b60405180910390f35b6100f66102c3565b6040516101039190610cab565b60405180910390f35b61012660048036038101906101219190610cc4565b6102cc565b6040516101339190610c83565b60405180910390f35b6101446102dc565b6040516101519190610d2f565b60405180910390f35b610174600480360381019061016f9190610c2b565b6102e4565b005b610190600480360381019061018b9190610d48565b6102f2565b60405161019d9190610cab565b60405180910390f35b6101ae610303565b6040516101bb9190610b7a565b60405180910390f35b6101de60048036038101906101d99190610c2b565b610393565b6040516101eb9190610c83565b60405180910390f35b61020e60048036038101906102099190610d73565b6103a2565b60405161021b9190610cab565b60405180910390f35b60606003805461023390610dde565b80601f016020809104026020016040519081016040528092919081815260200182805461025f90610dde565b80156102aa5780601f10610281576101008083540402835291602001916102aa565b820191905f5260205f20905b81548152906001019060200180831161028d57829003601f168201915b5050505050905090565b5f6102bf83836103b5565b5f80f35b5f600254905090565b5f6102d88484846103d7565b5f80f35b5f6012905090565b6102ee8282610405565b5050565b5f6102fc82610484565b9050919050565b60606004805461031290610dde565b80601f016020809104026020016040519081016040528092919081815260200182805461033e90610dde565b80156103895780601f1061036057610100808354040283529160200191610389565b820191905f5260205f20905b81548152906001019060200180831161036c57829003601f168201915b5050505050905090565b5f61039e83836104c9565b5f80f35b5f6103ad83836104eb565b905092915050565b5f806103bf61056d565b90506103cc818585610574565b600191505092915050565b5f806103e161056d565b90506103ee858285610586565b6103f9858585610618565b60019150509392505050565b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610475575f6040517fec442f0500000000000000000000000000000000000000000000000000000000815260040161046c9190610e1d565b60405180910390fd5b6104805f8383610708565b5050565b5f805f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050919050565b5f806104d361056d565b90506104e0818585610618565b600191505092915050565b5f60015f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054905092915050565b5f33905090565b6105818383836001610921565b505050565b5f61059184846103a2565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146106125781811015610603578281836040517ffb8f41b20000000000000000000000000000000000000000000000000000000081526004016105fa93929190610e36565b60405180910390fd5b61061184848484035f610921565b5b50505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610688575f6040517f96c6fd1e00000000000000000000000000000000000000000000000000000000815260040161067f9190610e1d565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16036106f8575f6040517fec442f050000000000000000000000000000000000000000000000000000000081526004016106ef9190610e1d565b60405180910390fd5b610703838383610708565b505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610758578060025f82825461074c9190610e98565b92505081905550610826565b5f805f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050818110156107e1578381836040517fe450d38c0000000000000000000000000000000000000000000000000000000081526004016107d893929190610e36565b60405180910390fd5b8181035f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550505b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361086d578060025f82825403925050819055506108b7565b805f808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516109149190610cab565b60405180910390a3505050565b5f73ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1603610991575f6040517fe602df050000000000000000000000000000000000000000000000000000000081526004016109889190610e1d565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610a01575f6040517f94280d620000000000000000000000000000000000000000000000000000000081526004016109f89190610e1d565b60405180910390fd5b8160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508015610aea578273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92584604051610ae19190610cab565b60405180910390a35b50505050565b5f81519050919050565b5f82825260208201905092915050565b5f5b83811015610b27578082015181840152602081019050610b0c565b5f8484015250505050565b5f601f19601f8301169050919050565b5f610b4c82610af0565b610b568185610afa565b9350610b66818560208601610b0a565b610b6f81610b32565b840191505092915050565b5f6020820190508181035f830152610b928184610b42565b905092915050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610bc782610b9e565b9050919050565b610bd781610bbd565b8114610be1575f80fd5b50565b5f81359050610bf281610bce565b92915050565b5f819050919050565b610c0a81610bf8565b8114610c14575f80fd5b50565b5f81359050610c2581610c01565b92915050565b5f8060408385031215610c4157610c40610b9a565b5b5f610c4e85828601610be4565b9250506020610c5f85828601610c17565b9150509250929050565b5f8115159050919050565b610c7d81610c69565b82525050565b5f602082019050610c965f830184610c74565b92915050565b610ca581610bf8565b82525050565b5f602082019050610cbe5f830184610c9c565b92915050565b5f805f60608486031215610cdb57610cda610b9a565b5b5f610ce886828701610be4565b9350506020610cf986828701610be4565b9250506040610d0a86828701610c17565b9150509250925092565b5f60ff82169050919050565b610d2981610d14565b82525050565b5f602082019050610d425f830184610d20565b92915050565b5f60208284031215610d5d57610d5c610b9a565b5b5f610d6a84828501610be4565b91505092915050565b5f8060408385031215610d8957610d88610b9a565b5b5f610d9685828601610be4565b9250506020610da785828601610be4565b9150509250929050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f6002820490506001821680610df557607f821691505b602082108103610e0857610e07610db1565b5b50919050565b610e1781610bbd565b82525050565b5f602082019050610e305f830184610e0e565b92915050565b5f606082019050610e495f830186610e0e565b610e566020830185610c9c565b610e636040830184610c9c565b949350505050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f610ea282610bf8565b9150610ead83610bf8565b9250828201905080821115610ec557610ec4610e6b565b5b9291505056fea2646970667358221220b88d95169fcafa24f302904c1d9f60a9d0d8b37907b43a0ef69b85ef238c60fd64736f6c63430008150033")]
    // SPDX-License-Identifier: MIT
    contract ERC20NoReturnMock is ERC20 {
        constructor() ERC20("ERC20NoReturnMock", "NRM") {}

        function transfer(address to, uint256 amount) public override returns (bool) {
            super.transfer(to, amount);
            assembly {
                return(0, 0)
            }
        }

        function transferFrom(address from, address to, uint256 amount) public override returns (bool) {
            super.transferFrom(from, to, amount);
            assembly {
                return(0, 0)
            }
        }

        function approve(address spender, uint256 amount) public override returns (bool) {
            super.approve(spender, amount);
            assembly {
                return(0, 0)
            }
        }

        function balanceOf(address account) public override view returns (uint256) {
            return super.balanceOf(account);
        }

        function mint(address account, uint256 value) public {
            super._mint(account, value);
        }

        function allowance(address owner, address spender) public view override returns (uint256) {
            return super.allowance(owner, spender);
        }
    }
}

pub async fn deploy(wallet: &Wallet) -> eyre::Result<Address> {
    // Deploy the contract.
    let contract = ERC20NoReturnMock::deploy(wallet).await?;
    Ok(*contract.address())
}