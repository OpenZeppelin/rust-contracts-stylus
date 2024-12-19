// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc4262Example {
    address private immutable _asset;
    uint8 private immutable _underlyingDecimals;
    
    constructor(address asset_) {
        (bool success, uint8 assetDecimals) = _tryGetAssetDecimals();
        _underlyingDecimals = success ? assetDecimals : 18;
        _asset = asset_;
    }

    function _tryGetAssetDecimals() private pure returns (bool ok, uint8 assetDecimals) {
       return (true, 0);
    }
}
