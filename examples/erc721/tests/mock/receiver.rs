#![allow(dead_code)]
#![cfg(feature = "e2e")]
use alloy::{
    primitives::{fixed_bytes, Address},
    sol,
};
use e2e::Wallet;

sol! {
    #[allow(missing_docs)]
    // Built with Remix IDE; solc v0.8.21+commit.d9974bed
    #[sol(rpc, bytecode="60c060405234801561000f575f80fd5b5060405161093438038061093483398181016040528101906100319190610126565b817bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19166080817bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168152505080600481111561008a57610089610164565b5b60a081600481111561009f5761009e610164565b5b815250505050610191565b5f80fd5b5f7fffffffff0000000000000000000000000000000000000000000000000000000082169050919050565b6100e2816100ae565b81146100ec575f80fd5b50565b5f815190506100fd816100d9565b92915050565b6005811061010f575f80fd5b50565b5f8151905061012081610103565b92915050565b5f806040838503121561013c5761013b6100aa565b5b5f610149858286016100ef565b925050602061015a85828601610112565b9150509250929050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602160045260245ffd5b60805160a0516107686101cc5f395f818160740152818160c40152818161014b01526101f301525f8181610183015261027801526107685ff3fe608060405234801561000f575f80fd5b5060043610610029575f3560e01c8063150b7a021461002d575b5f80fd5b6100476004803603810190610042919061047b565b61005d565b6040516100549190610535565b60405180910390f35b5f600160048111156100725761007161054e565b5b7f000000000000000000000000000000000000000000000000000000000000000060048111156100a5576100a461054e565b5b036100ae575f80fd5b600260048111156100c2576100c161054e565b5b7f000000000000000000000000000000000000000000000000000000000000000060048111156100f5576100f461054e565b5b03610135576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161012c906105d5565b60405180910390fd5b600360048111156101495761014861054e565b5b7f0000000000000000000000000000000000000000000000000000000000000000600481111561017c5761017b61054e565b5b036101de577f00000000000000000000000000000000000000000000000000000000000000006040517f66435bc00000000000000000000000000000000000000000000000000000000081526004016101d59190610535565b60405180910390fd5b6004808111156101f1576101f061054e565b5b7f000000000000000000000000000000000000000000000000000000000000000060048111156102245761022361054e565b5b0361023a575f805f6102369190610620565b9050505b7ed9411ae77b2bacabe5cbe62a2abdbeb78992a0182c6f3c83e0029c7615d6b68585858560405161026e94939291906106e8565b60405180910390a17f00000000000000000000000000000000000000000000000000000000000000009050949350505050565b5f604051905090565b5f80fd5b5f80fd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6102db826102b2565b9050919050565b6102eb816102d1565b81146102f5575f80fd5b50565b5f81359050610306816102e2565b92915050565b5f819050919050565b61031e8161030c565b8114610328575f80fd5b50565b5f8135905061033981610315565b92915050565b5f80fd5b5f80fd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b61038d82610347565b810181811067ffffffffffffffff821117156103ac576103ab610357565b5b80604052505050565b5f6103be6102a1565b90506103ca8282610384565b919050565b5f67ffffffffffffffff8211156103e9576103e8610357565b5b6103f282610347565b9050602081019050919050565b828183375f83830152505050565b5f61041f61041a846103cf565b6103b5565b90508281526020810184848401111561043b5761043a610343565b5b6104468482856103ff565b509392505050565b5f82601f8301126104625761046161033f565b5b813561047284826020860161040d565b91505092915050565b5f805f8060808587031215610493576104926102aa565b5b5f6104a0878288016102f8565b94505060206104b1878288016102f8565b93505060406104c28782880161032b565b925050606085013567ffffffffffffffff8111156104e3576104e26102ae565b5b6104ef8782880161044e565b91505092959194509250565b5f7fffffffff0000000000000000000000000000000000000000000000000000000082169050919050565b61052f816104fb565b82525050565b5f6020820190506105485f830184610526565b92915050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52602160045260245ffd5b5f82825260208201905092915050565b7f45524337323152656365697665724d6f636b3a20726576657274696e670000005f82015250565b5f6105bf601d8361057b565b91506105ca8261058b565b602082019050919050565b5f6020820190508181035f8301526105ec816105b3565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601260045260245ffd5b5f61062a8261030c565b91506106358361030c565b925082610645576106446105f3565b5b828204905092915050565b610659816102d1565b82525050565b6106688161030c565b82525050565b5f81519050919050565b5f82825260208201905092915050565b5f5b838110156106a557808201518184015260208101905061068a565b5f8484015250505050565b5f6106ba8261066e565b6106c48185610678565b93506106d4818560208601610688565b6106dd81610347565b840191505092915050565b5f6080820190506106fb5f830187610650565b6107086020830186610650565b610715604083018561065f565b818103606083015261072781846106b0565b90509594505050505056fea2646970667358221220b3015e4cfe9292167a6c435826958084eb394c9609b38940a0899ffb42eaa2df64736f6c63430008150033")]
    contract ERC721ReceiverMock is IERC721Receiver {
        enum RevertType {
            None,
            RevertWithoutMessage,
            RevertWithMessage,
            RevertWithCustomError,
            Panic
        }

        bytes4 private immutable _retval;
        RevertType private immutable _error;

        #[derive(Debug, PartialEq)]
        event Received(address operator, address from, uint256 tokenId, bytes data);

        error CustomError(bytes4);

        constructor(bytes4 retval, RevertType error) {
            _retval = retval;
            _error = error;
        }

        function onERC721Received(
            address operator,
            address from,
            uint256 tokenId,
            bytes memory data
        ) public returns (bytes4) {
            if (_error == RevertType.RevertWithoutMessage) {
                revert();
            } else if (_error == RevertType.RevertWithMessage) {
                revert("ERC721ReceiverMock: reverting");
            } else if (_error == RevertType.RevertWithCustomError) {
                revert CustomError(_retval);
            } else if (_error == RevertType.Panic) {
                uint256 a = uint256(0) / uint256(0);
                a;
            }

            emit Received(operator, from, tokenId, data);
            return _retval;
        }
    }
}

pub async fn deploy(
    wallet: &Wallet,
    error: ERC721ReceiverMock::RevertType,
) -> eyre::Result<Address> {
    let retval = fixed_bytes!("150b7a02");

    // Deploy the contract.
    let contract = ERC721ReceiverMock::deploy(wallet, retval, error).await?;
    Ok(*contract.address())
}
