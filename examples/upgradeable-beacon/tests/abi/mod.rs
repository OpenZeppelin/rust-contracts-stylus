#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract UpgradeableBeaconExample {
        function implementation() public view returns (address implementation);
        function upgradeTo(address newImplementation) public;
        function owner() public view returns (address owner);
        function transferOwnership(address newOwner) public;
        function renounceOwnership() public;

        error BeaconInvalidImplementation(address implementation);
        error OwnableUnauthorizedAccount(address sender);
        error OwnableInvalidOwner(address owner);

        #[derive(Debug, PartialEq)]
        event Upgraded(address indexed implementation);
        #[derive(Debug, PartialEq)]
        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    }
);
