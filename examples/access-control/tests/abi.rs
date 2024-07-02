#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract AccessControl {
        constructor();

        function hasRole(bytes32 role, address account) public view virtual returns (bool hasRole);
        function getRoleAdmin(bytes32 role) public view virtual returns (bytes32);
        function grantRole(bytes32 role, address account) public virtual;
        function revokeRole(bytes32 role, address account) public virtual;
        function renounceRole(bytes32 role, address callerConfirmation) public virtual;

        error AccessControlUnauthorizedAccount(address account, bytes32 neededRole);
        error AccessControlBadConfirmation();

        #[derive(Debug, PartialEq)]
        event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole);
        #[derive(Debug, PartialEq)]
        event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender);
        #[derive(Debug, PartialEq)]
        event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender);
    }
);
