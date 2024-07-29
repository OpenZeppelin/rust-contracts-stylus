#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Crypto {
        error ECDSAInvalidSignature();
        error ECDSAInvalidSignatureS(bytes32 s);

        #[derive(Debug)]
        function recover(bytes32 hash, uint8 v, bytes32 r, bytes32 s) internal pure returns (address recovered);
    }
);
