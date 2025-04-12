// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract AccessControlEnumerableExample {
    // ERC20 storage
    mapping(address => uint256) _balances;
    mapping(address => mapping(address => uint256)) _allowances;
    uint256 _totalSupply;
    string _name;
    string _symbol;
    uint8 _decimals;

    // AccessControl storage
    struct RoleData {
        mapping(address account => bool) hasRole;
        bytes32 adminRole;
    }
    mapping(bytes32 role => RoleData) private _roles;

    // AccessControlEnumerable storage
    mapping(bytes32 role => address[]) private _roleMembers;

    bytes32 public constant DEFAULT_ADMIN_ROLE = 0x00;
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant BURNER_ROLE = keccak256("BURNER_ROLE");

    event RoleGranted(
        bytes32 indexed role,
        address indexed account,
        address indexed sender
    );

    constructor(string memory name_, string memory symbol_) {
        _name = name_;
        _symbol = symbol_;
        _decimals = 18;

        // Grant the deployer the default admin role
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function _grantRole(bytes32 role, address account) internal returns (bool) {
        if (!_roles[role].hasRole[account]) {
            _roles[role].hasRole[account] = true;
            _roleMembers[role].push(account);
            emit RoleGranted(role, account, msg.sender);
            return true;
        }
        return false;
    }
} 