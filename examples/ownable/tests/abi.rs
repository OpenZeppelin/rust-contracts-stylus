#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Ownable {
        error OwnableUnauthorizedAccount(address account);
        error OwnableInvalidOwner(address owner);

        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

        function owner() public view virtual returns (address);
        function renounceOwnership() public virtual onlyOwner;
        function transferOwnership(address newOwner) public virtual onlyOwner;
    }
);
