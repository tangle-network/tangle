// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title Oracle Precompile Interface
/// @notice Interface for interacting with the Oracle pallet through EVM
interface Oracle {
    /// @notice Feed values into the oracle
    /// @param keys Array of oracle keys
    /// @param values Array of corresponding values for the keys
    /// @dev The length of keys and values must match
    /// @return success True if the operation was successful
    function feed_values(uint256[] calldata keys, uint256[] calldata values) external returns (bool success);

    /// @notice Get the value for a specific key from the oracle
    /// @param key The oracle key to query
    /// @return value The value associated with the key
    /// @return timestamp The timestamp when the value was last updated
    function get_value(uint256 key) external view returns (uint256 value, uint256 timestamp);

    /// @notice Emitted when new values are fed into the oracle
    /// @param operator The account that fed the values
    /// @param keys Array of oracle keys that were updated
    /// @param values Array of values that were fed
    event ValuesFed(address indexed operator, uint256[] keys, uint256[] values);

    /// @notice Emitted when a value is retrieved from the oracle
    /// @param key The oracle key that was queried
    /// @param value The value that was retrieved
    /// @param timestamp The timestamp of when the value was last updated
    event ValueRetrieved(uint256 indexed key, uint256 value, uint256 timestamp);
}
