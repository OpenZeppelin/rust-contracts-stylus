#![allow(dead_code)]
#![cfg(feature = "e2e")]
use alloy::{primitives::Address, sol};
use e2e::Wallet;

sol! {
    #[allow(missing_docs)]
    // Built with Remix IDE; solc v0.8.24+commit.e11b9ed9
    #[sol(rpc, bytecode="608060405234801562000010575f80fd5b506040518060400160405280601881526020017f45524332304661696c696e675472616e736665724d6f636b00000000000000008152506040518060400160405280600381526020017f52464d000000000000000000000000000000000000000000000000000000000081525081600390816200008e91906200030d565b508060049081620000a091906200030d565b505050620003f1565b5f81519050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f60028204905060018216806200012557607f821691505b6020821081036200013b576200013a620000e0565b5b50919050565b5f819050815f5260205f209050919050565b5f6020601f8301049050919050565b5f82821b905092915050565b5f600883026200019f7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8262000162565b620001ab868362000162565b95508019841693508086168417925050509392505050565b5f819050919050565b5f819050919050565b5f620001f5620001ef620001e984620001c3565b620001cc565b620001c3565b9050919050565b5f819050919050565b6200021083620001d5565b620002286200021f82620001fc565b8484546200016e565b825550505050565b5f90565b6200023e62000230565b6200024b81848462000205565b505050565b5b818110156200027257620002665f8262000234565b60018101905062000251565b5050565b601f821115620002c1576200028b8162000141565b620002968462000153565b81016020851015620002a6578190505b620002be620002b58562000153565b83018262000250565b50505b505050565b5f82821c905092915050565b5f620002e35f1984600802620002c6565b1980831691505092915050565b5f620002fd8383620002d2565b9150826002028217905092915050565b6200031882620000a9565b67ffffffffffffffff811115620003345762000333620000b3565b5b6200034082546200010d565b6200034d82828562000276565b5f60209050601f83116001811462000383575f84156200036e578287015190505b6200037a8582620002f0565b865550620003e9565b601f198416620003938662000141565b5f5b82811015620003bc5784890151825560018201915060208501945060208101905062000395565b86831015620003dc5784890151620003d8601f891682620002d2565b8355505b6001600288020188555050505b505050505050565b610f1780620003ff5f395ff3fe608060405234801561000f575f80fd5b50600436106100a7575f3560e01c806340c10f191161006f57806340c10f191461016557806370a08231146101815780638483acfe146101b157806395d89b41146101cd578063a9059cbb146101eb578063dd62ed3e1461021b576100a7565b806306fdde03146100ab578063095ea7b3146100c957806318160ddd146100f957806323b872dd14610117578063313ce56714610147575b5f80fd5b6100b361024b565b6040516100c09190610b90565b60405180910390f35b6100e360048036038101906100de9190610c41565b6102db565b6040516100f09190610c99565b60405180910390f35b6101016102ee565b60405161010e9190610cc1565b60405180910390f35b610131600480360381019061012c9190610cda565b6102f7565b60405161013e9190610c99565b60405180910390f35b61014f61030c565b60405161015c9190610d45565b60405180910390f35b61017f600480360381019061017a9190610c41565b610314565b005b61019b60048036038101906101969190610d5e565b610322565b6040516101a89190610cc1565b60405180910390f35b6101cb60048036038101906101c69190610cda565b610333565b005b6101d5610343565b6040516101e29190610b90565b60405180910390f35b61020560048036038101906102009190610c41565b6103d3565b6040516102129190610c99565b60405180910390f35b61023560048036038101906102309190610d89565b6103da565b6040516102429190610cc1565b60405180910390f35b60606003805461025a90610df4565b80601f016020809104026020016040519081016040528092919081815260200182805461028690610df4565b80156102d15780601f106102a8576101008083540402835291602001916102d1565b820191905f5260205f20905b8154815290600101906020018083116102b457829003601f168201915b5050505050905090565b5f6102e683836103ed565b905092915050565b5f600254905090565b5f61030384848461040f565b90509392505050565b5f6012905090565b61031e828261043d565b5050565b5f61032c826104bc565b9050919050565b61033e838383610501565b505050565b60606004805461035290610df4565b80601f016020809104026020016040519081016040528092919081815260200182805461037e90610df4565b80156103c95780601f106103a0576101008083540402835291602001916103c9565b820191905f5260205f20905b8154815290600101906020018083116103ac57829003601f168201915b5050505050905090565b5f92915050565b5f6103e58383610513565b905092915050565b5f806103f7610595565b9050610404818585610501565b600191505092915050565b5f80610419610595565b905061042685828561059c565b61043185858561062e565b60019150509392505050565b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16036104ad575f6040517fec442f050000000000000000000000000000000000000000000000000000000081526004016104a49190610e33565b60405180910390fd5b6104b85f838361071e565b5050565b5f805f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050919050565b61050e8383836001610937565b505050565b5f60015f8473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2054905092915050565b5f33905090565b5f6105a784846103da565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146106285781811015610619578281836040517ffb8f41b200000000000000000000000000000000000000000000000000000000815260040161061093929190610e4c565b60405180910390fd5b61062784848484035f610937565b5b50505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361069e575f6040517f96c6fd1e0000000000000000000000000000000000000000000000000000000081526004016106959190610e33565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361070e575f6040517fec442f050000000000000000000000000000000000000000000000000000000081526004016107059190610e33565b60405180910390fd5b61071983838361071e565b505050565b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff160361076e578060025f8282546107629190610eae565b9250508190555061083c565b5f805f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20549050818110156107f7578381836040517fe450d38c0000000000000000000000000000000000000000000000000000000081526004016107ee93929190610e4c565b60405180910390fd5b8181035f808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f2081905550505b5f73ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610883578060025f82825403925050819055506108cd565b805f808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f82825401925050819055505b8173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161092a9190610cc1565b60405180910390a3505050565b5f73ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16036109a7575f6040517fe602df0500000000000000000000000000000000000000000000000000000000815260040161099e9190610e33565b60405180910390fd5b5f73ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610a17575f6040517f94280d62000000000000000000000000000000000000000000000000000000008152600401610a0e9190610e33565b60405180910390fd5b8160015f8673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f205f8573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020015f20819055508015610b00578273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92584604051610af79190610cc1565b60405180910390a35b50505050565b5f81519050919050565b5f82825260208201905092915050565b5f5b83811015610b3d578082015181840152602081019050610b22565b5f8484015250505050565b5f601f19601f8301169050919050565b5f610b6282610b06565b610b6c8185610b10565b9350610b7c818560208601610b20565b610b8581610b48565b840191505092915050565b5f6020820190508181035f830152610ba88184610b58565b905092915050565b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f610bdd82610bb4565b9050919050565b610bed81610bd3565b8114610bf7575f80fd5b50565b5f81359050610c0881610be4565b92915050565b5f819050919050565b610c2081610c0e565b8114610c2a575f80fd5b50565b5f81359050610c3b81610c17565b92915050565b5f8060408385031215610c5757610c56610bb0565b5b5f610c6485828601610bfa565b9250506020610c7585828601610c2d565b9150509250929050565b5f8115159050919050565b610c9381610c7f565b82525050565b5f602082019050610cac5f830184610c8a565b92915050565b610cbb81610c0e565b82525050565b5f602082019050610cd45f830184610cb2565b92915050565b5f805f60608486031215610cf157610cf0610bb0565b5b5f610cfe86828701610bfa565b9350506020610d0f86828701610bfa565b9250506040610d2086828701610c2d565b9150509250925092565b5f60ff82169050919050565b610d3f81610d2a565b82525050565b5f602082019050610d585f830184610d36565b92915050565b5f60208284031215610d7357610d72610bb0565b5b5f610d8084828501610bfa565b91505092915050565b5f8060408385031215610d9f57610d9e610bb0565b5b5f610dac85828601610bfa565b9250506020610dbd85828601610bfa565b9150509250929050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602260045260245ffd5b5f6002820490506001821680610e0b57607f821691505b602082108103610e1e57610e1d610dc7565b5b50919050565b610e2d81610bd3565b82525050565b5f602082019050610e465f830184610e24565b92915050565b5f606082019050610e5f5f830186610e24565b610e6c6020830185610cb2565b610e796040830184610cb2565b949350505050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f610eb882610c0e565b9150610ec383610c0e565b9250828201905080821115610edb57610eda610e81565b5b9291505056fea264697066735822122067beaed7a6a24d711579cadb98788abdcb4406e94f02fba0b0b2e7a7a5e78eee64736f6c63430008180033")]
    // SPDX-License-Identifier: MIT
    contract ERC20FailingTransferMock is ERC20 {
        constructor() ERC20("ERC20FailingTransferMock", "RFM") {}

        function approve(address spender, uint256 value)
            public
            override
            returns (bool)
        {
            return super.approve(spender, value);
        }

        // WARNING: This code is for testing purposes only! Do not use in production.
        function regular_approve(
            address owner,
            address spender,
            uint256 amount
        ) public {
            super._approve(owner, spender, amount);
        }

        function transfer(address, uint256) public override returns (bool) {
            return false;
        }

        function transferFrom(
            address from,
            address to,
            uint256 value
        ) public override returns (bool) {
            return super.transferFrom(from, to, value);
        }

        function balanceOf(address account) public view override returns (uint256) {
            return super.balanceOf(account);
        }

        function mint(address account, uint256 value) public {
            super._mint(account, value);
        }

        function allowance(address owner, address spender)
            public
            view
            override
            returns (uint256)
        {
            return super.allowance(owner, spender);
        }
    }
}

pub async fn deploy(wallet: &Wallet) -> eyre::Result<Address> {
    // Deploy the contract.
    let contract = ERC20FailingTransferMock::deploy(wallet).await?;
    Ok(*contract.address())
}
