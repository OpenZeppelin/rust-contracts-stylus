#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Ownable {
        error OwnableUnauthorizedAccount(address account);
        error OwnableInvalidOwner(address owner);

        #[derive(Debug, PartialEq)]
        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

        function owner() public view virtual returns (address owner);
        function renounceOwnership() public virtual onlyOwner;
        function transferOwnership(address newOwner) public virtual;
    }
);
