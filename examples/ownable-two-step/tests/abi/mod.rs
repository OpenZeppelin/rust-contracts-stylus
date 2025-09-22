#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract Ownable2Step {
        error OwnableUnauthorizedAccount(address account);
        error OwnableInvalidOwner(address owner);

        #[derive(Debug, PartialEq)]
        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
        #[derive(Debug, PartialEq)]
        event OwnershipTransferStarted(address indexed previousOwner, address indexed newOwner);

        function owner() public view virtual returns (address owner);
        function pendingOwner() public view returns (address pendingOwner);
        function renounceOwnership() public virtual onlyOwner;
        function transferOwnership(address newOwner) public virtual;
        function acceptOwnership() public virtual;
    }
);
