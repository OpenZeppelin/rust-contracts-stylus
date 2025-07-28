#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract PrecompilesExample {
        error ECDSAInvalidSignature();
        error ECDSAInvalidSignatureS(bytes32 s);

        #[derive(Debug)]
        function recover(bytes32 hash, uint8 v, bytes32 r, bytes32 s) pure returns (address recovered);
        #[derive(Debug)]
        function testP256Verify(bytes32 hash, bytes32 r, bytes32 s, bytes32 x, bytes32 y) pure returns (bool result);
    }
);
