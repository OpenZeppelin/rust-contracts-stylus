#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract PoseidonExample {
        #[derive(Debug)]
        function hash(bytes calldata data) external view returns (bytes32 hash);
    }
);
