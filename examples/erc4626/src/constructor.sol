// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc4626Example {
    address private immutable _asset;
    uint8 private immutable _underlyingDecimals;
    
    constructor(address asset_) {
        (bool success, uint8 assetDecimals) = _tryGetAssetDecimals();
        _underlyingDecimals = success ? assetDecimals : 18;
        _asset = asset_;
    }

    constructor(IERC20 asset_) {
        (bool success, uint8 assetDecimals) = _tryGetAssetDecimals(asset_);
        _underlyingDecimals = success ? assetDecimals : 18;
        _asset = asset_;
    }

    /**
     * @dev Attempts to fetch the asset decimals. A return value of false indicates that the attempt failed in some way.
     */
    function _tryGetAssetDecimals(IERC20 asset_) private view returns (bool ok, uint8 assetDecimals) {
        (bool success, bytes memory encodedDecimals) = address(asset_).staticcall(
            abi.encodeCall(IERC20Metadata.decimals, ())
        );
        if (success && encodedDecimals.length >= 32) {
            uint256 returnedDecimals = abi.decode(encodedDecimals, (uint256));
            if (returnedDecimals <= type(uint8).max) {
                return (true, uint8(returnedDecimals));
            }
        }
        return (false, 0);
    }
}
