#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Crypto {
        error ECDSAInvalidSignature();
        error ECDSAInvalidSignatureLength(uint256 length);
        error ECDSAInvalidSignatureS(bytes32 s);

        #[derive(Debug)]
        function recover(bytes32 hash, bytes memory signature) internal pure returns (address recovered);
        #[derive(Debug)]
        function recover(bytes32 hash, uint8 v, bytes32 r, bytes32 s) internal pure returns (address recovered);
    }
);
