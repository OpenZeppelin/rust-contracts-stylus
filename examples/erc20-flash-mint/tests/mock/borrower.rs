#![allow(dead_code)]
#![cfg(feature = "e2e")]
use alloy::{primitives::Address, sol};
use e2e::Wallet;

sol! {
    #[allow(missing_docs)]
    // Built with solc v0.8.24; solc examples/erc20-flash-mint/src/ERC3156FlashBorrowerMock.sol --via-ir --optimize --bin
    #[sol(rpc, bytecode="60803461007e57601f6102cc38819003918201601f19168301916001600160401b0383118484101761008257808492604094855283398101031261007e57610052602061004b83610096565b9201610096565b15159060ff61ff005f5492151560081b1692169061ffff191617175f5560405161022890816100a48239f35b5f80fd5b634e487b7160e01b5f52604160045260245ffd5b5190811515820361007e5756fe6080604081815260049182361015610015575f80fd5b5f3560e01c6323e30c8b14610028575f80fd5b346101905760a0366003190112610190576001600160a01b0390833582811603610190576024359182168092036101905767ffffffffffffffff90608435828111610190573660238201121561019057808601358381116101905736910160240111610190578233036101b9575082516370a0823160e01b815230858201526020948582602481875afa9283156101af575f93610134575b5050507f6ff2acfcb07917b1e80e53f0fe390b467b1151d15b38730a6e08397799c05a8b916060918451918252308683015284820152a15f5460081c60ff161561012d577f439148f0bbc682ca079e46d6e2c2f0c1e3b820f1a291b069d8882abf8cf18dd9905b51908152f35b5f90610127565b9091925085913d87116101a7575b601f8301601f191684019182118483101761019457508591839186528101031261019057518160607f6ff2acfcb07917b1e80e53f0fe390b467b1151d15b38730a6e08397799c05a8b6100c0565b5f80fd5b604190634e487b7160e01b5f525260245ffd5b3d9250610142565b85513d5f823e3d90fd5b62461bcd60e51b81526020858201526015602482015274496e76616c696420746f6b656e206164647265737360581b6044820152606490fdfea2646970667358221220a48b244fa593dff78d6e702df9beea12adb5701cd0074d43b4feb10e3096f78464736f6c63430008180033")]
    contract ERC3156FlashBorrowerMock is IERC3156FlashBorrower {
        bytes32 internal constant _RETURN_VALUE =
            keccak256("ERC3156FlashBorrower.onFlashLoan");

        bool _enableApprove;
        bool _enableReturn;

        #[derive(Debug, PartialEq)]
        event BalanceOf(address token, address account, uint256 value);
        #[derive(Debug, PartialEq)]
        event TotalSupply(address token, uint256 value);

        constructor(bool enableReturn, bool enableApprove) {
            _enableApprove = enableApprove;
            _enableReturn = enableReturn;
        }

        function onFlashLoan(
            address /* initiator */,
            address token,
            uint256 amount,
            uint256 fee,
            bytes calldata data
        ) public returns (bytes32) {
            require(msg.sender == token, "Invalid token address");

            emit BalanceOf(
                token,
                address(this),
                IERC20(token).balanceOf(address(this))
            );

            // emit TotalSupply(token, IERC20(token).totalSupply());

            // if (data.length > 0) {
            //     // WARNING: This code is for testing purposes only! Do not use in production.
            //     Address.functionCall(token, data);
            // }

            // if (_enableApprove) {
            //     IERC20(token).approve(token, amount + fee);
            // }

            return _enableReturn ? _RETURN_VALUE : bytes32(0);
        }
    }
}

pub async fn deploy(
    wallet: &Wallet,
    enable_return: bool,
    enable_approve: bool,
) -> eyre::Result<Address> {
    // Deploy the contract.
    let contract =
        ERC3156FlashBorrowerMock::deploy(wallet, enable_return, enable_approve)
            .await?;
    Ok(*contract.address())
}
