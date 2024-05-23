pragma solidity ^0.8.21;

contract Counter {
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256))
        private _allowances;
    uint256 private _totalSupply;
    string private _name;
    string private _symbol;
    uint256 private _cap;
    bool private _paused;

    error ERC20InvalidCap(uint256 cap);

    constructor(string memory name_, string memory symbol_, uint256 cap_) {
        _name = name_;
        _symbol = symbol_;

        if (cap_ == 0) {
            revert ERC20InvalidCap(0);
        }

        _cap = cap_;
        _paused = false;
    }

    function cap() public view virtual returns (uint256) {
        return _cap;
    }
}
