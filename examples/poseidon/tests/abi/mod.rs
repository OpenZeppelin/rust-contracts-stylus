#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract PoseidonExample {
        #[derive(Debug)]
        function hash(uint[2] memory inputs) external view returns (uint hash);
    }
);
