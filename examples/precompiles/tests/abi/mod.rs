#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract PrecompilesExample {
        error ECDSAInvalidSignature();
        error ECDSAInvalidSignatureS(bytes32 s);

        #[derive(Debug)]
        function recover(bytes32 hash, uint8 v, bytes32 r, bytes32 s) public pure returns (address recovered);

        #[derive(Debug)]
        function callBls12G1Add(bytes16 a, bytes16 b) public pure returns (bytes16 result);
    }
);
