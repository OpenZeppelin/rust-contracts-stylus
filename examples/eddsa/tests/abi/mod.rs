#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract EddsaExample {
        #[derive(Debug)]
        function sign(uint secret_key, bytes calldata message) external view returns (bytes signature);
    }
);
