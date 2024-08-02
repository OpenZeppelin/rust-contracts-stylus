// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

contract Erc721ConsecutiveExample {
    mapping(uint256 tokenId => address) private _owners;
    mapping(address owner => uint256) private _balances;
    mapping(uint256 tokenId => address) private _tokenApprovals;
    mapping(address owner => mapping(address operator => bool)) private _operatorApprovals;

    Checkpoint160[] private _checkpoints; // _sequentialOwnership
    mapping(uint256 bucket => uint256) private _data; // _sequentialBurn
    bool private _initialized;

    error ERC721InvalidReceiver(address receiver);
    error ERC721ForbiddenBatchMint();
    error ERC721ExceededMaxBatchMint(uint256 batchSize, uint256 maxBatch);
    error ERC721ForbiddenMint();
    error ERC721ForbiddenBatchBurn();
    error CheckpointUnorderedInsertion();

    event ConsecutiveTransfer(
        uint256 indexed fromTokenId,
        uint256 toTokenId,
        address indexed fromAddress,
        address indexed toAddress
    );

    struct Checkpoint160 {
        uint96 _key;
        uint160 _value;
    }

    constructor(address[] memory receivers, uint96[] memory amounts) {
        for (uint256 i = 0; i < receivers.length; ++i) {
            _mintConsecutive(receivers[i], amounts[i]);
        }
        _initialized = true;
    }

    function latestCheckpoint() internal view returns (bool exists, uint96 _key, uint160 _value) {
        uint256 pos = _checkpoints.length;
        if (pos == 0) {
            return (false, 0, 0);
        } else {
            Checkpoint160 storage ckpt = _checkpoints[pos - 1];
            return (true, ckpt._key, ckpt._value);
        }
    }

    function push(uint96 key, uint160 value) internal returns (uint160, uint160) {
        return _insert(key, value);
    }

    function _insert(uint96 key, uint160 value) private returns (uint160, uint160) {
        uint256 pos = _checkpoints.length;

        if (pos > 0) {
            Checkpoint160 storage last = _checkpoints[pos - 1];
            uint96 lastKey = last._key;
            uint160 lastValue = last._value;

            // Checkpoint keys must be non-decreasing.
            if (lastKey > key) {
                revert CheckpointUnorderedInsertion();
            }

            // Update or push new checkpoint.
            if (lastKey == key) {
                _checkpoints[pos - 1]._value = value;
            } else {
                _checkpoints.push(Checkpoint160({_key: key, _value: value}));
            }
            return (lastValue, value);
        } else {
            _checkpoints.push(Checkpoint160({_key: key, _value: value}));
            return (0, value);
        }
    }

    function _mintConsecutive(address to, uint96 batchSize) internal virtual returns (uint96) {
        uint96 next = _nextConsecutiveId();

        // minting a batch of size 0 is a no-op.
        if (batchSize > 0) {
            if (address(this).code.length > 0) {
                revert ERC721ForbiddenBatchMint();
            }
            if (to == address(0)) {
                revert ERC721InvalidReceiver(address(0));
            }

            uint256 maxBatchSize = _maxBatchSize();
            if (batchSize > maxBatchSize) {
                revert ERC721ExceededMaxBatchMint(batchSize, maxBatchSize);
            }

            // push an ownership checkpoint & emit event.
            uint96 last = next + batchSize - 1;
            push(last, uint160(to));

            // The invariant required by this function is preserved because the new sequentialOwnership checkpoint
            // is attributing ownership of `batchSize` new tokens to account `to`.
            _increaseBalance(to, batchSize);

            emit ConsecutiveTransfer(next, last, address(0), to);
        }

        return next;
    }

    function _nextConsecutiveId() private view returns (uint96) {
        (bool exists, uint96 latestId,) = latestCheckpoint();
        return exists ? latestId + 1 : _firstConsecutiveId();
    }

    // TODO#q: _firstConsecutiveId and _maxBatchSize are duplicated in here
    //  and inside generic params for the contract.
    //  Make sense to assign these values inside solidity constructor.
    function _firstConsecutiveId() internal view virtual returns (uint96) {
        return 0;
    }

    function _maxBatchSize() internal view virtual returns (uint96) {
        return 5000;
    }

    function _increaseBalance(address account, uint128 value) internal virtual {
        unchecked {
            _balances[account] += value;
        }
    }
}
