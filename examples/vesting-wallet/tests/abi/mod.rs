#![allow(dead_code)]
use alloy::sol;

sol!(
     #[sol(rpc)]
     contract VestingWallet {
          function owner() public view virtual returns (address owner);
          function start() external view returns (uint256 start);
          function duration() external view returns (uint256 duration);
          function end() external view returns (uint256 end);
          function released() external view returns (uint256 released);
          function released(address token) external view returns (uint256 released);
          function releasable() external view returns (uint256 releasable);
          function releasable(address token) external view returns (uint256 releasable);
          function release() external;
          function release(address token) external;
          function vestedAmount(uint64 timestamp) external view returns (uint256 vestedAmount);
          function vestedAmount(address token, uint64 timestamp) external view returns (uint256 vestedAmount);

          error OwnableUnauthorizedAccount(address account);
          error OwnableInvalidOwner(address owner);
          error ReleaseEtherFailed();
          error SafeErc20FailedOperation(address token);
          error InvalidToken(address token);

          #[derive(Debug, PartialEq)]
          event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
          #[derive(Debug, PartialEq)]
          event EtherReleased(uint256 amount);
          #[derive(Debug, PartialEq)]
          event ERC20Released(address indexed token, uint256 amount);
   }
);
