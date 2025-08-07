#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract EddsaExample {
        #[derive(Debug)]
        function verify(uint[2] verifying_key, uint[3] signature, bytes calldata message) external view returns (bool is_valid);
    }
);
