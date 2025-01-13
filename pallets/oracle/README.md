# Oracle Pallet

## Overview

The Oracle pallet provides a decentralized mechanism for feeding external off-chain data into the Tangle blockchain. It allows authorized oracle operators to submit data which can then be aggregated to provide reliable on-chain values.

## Features

- **Authorized Data Feeding**: Only authorized operators can submit data to the oracle
- **Timestamped Values**: All submitted values include timestamps for freshness verification
- **Data Aggregation**: Raw values from multiple operators can be combined using customizable aggregation logic
- **Flexible Key-Value Storage**: Supports arbitrary key types for versatile data storage
- **EVM Integration**: Includes precompile support for Ethereum compatibility

## Interface

### Extrinsics (Callable Functions)

- `feed_values(values: Vec<(OracleKey, OracleValue)>)`: Submit multiple key-value pairs to the oracle
  - Requires sender to be an authorized operator
  - Values are stored with current timestamp
  - Limited by `MaxFeedValues` configuration

### Storage Queries

- `get(key)`: Retrieve the current combined value for a given key
- `get_all_values()`: Retrieve all stored key-value pairs
- `read_raw_values(key)`: Get all raw values submitted for a specific key

### EVM Precompile Interface

The pallet includes an EVM precompile that exposes the following functions:

```solidity
interface IOraclePrecompile {
    // Feed multiple values into the oracle
    function feedValues(uint256[] keys, uint256[] values) external;
    
    // Read a value from the oracle
    function getValue(uint256 key) external view returns (uint256 value, uint256 timestamp);
}
```

## Configuration

The pallet is configurable through the following types:

- `OracleKey`: The type used for oracle keys (e.g., `u32`)
- `OracleValue`: The type used for oracle values (e.g., `u64`)
- `MaxFeedValues`: Maximum number of values that can be fed in a single call
- `Members`: Source of oracle operator membership (typically using `pallet-membership`)

## Usage Examples

### Feeding Values

```rust
// Submit values through runtime call
Oracle::feed_values(
    RuntimeOrigin::signed(operator_account),
    vec![(key1, value1), (key2, value2)]
)?;

// Read values
if let Some(timestamped_value) = Oracle::get(&key) {
    let value = timestamped_value.value;
    let timestamp = timestamped_value.timestamp;
    // Use the value...
}
```

### Through EVM

```solidity
// Assuming the precompile is deployed at ORACLE_ADDRESS
IOraclePrecompile oracle = IOraclePrecompile(ORACLE_ADDRESS);

// Feed values
uint256[] memory keys = new uint256[](2);
uint256[] memory values = new uint256[](2);
keys[0] = 1;
keys[1] = 2;
values[0] = 100;
values[1] = 200;
oracle.feedValues(keys, values);

// Read value
(uint256 value, uint256 timestamp) = oracle.getValue(1);
```

## Security

- Only authorized operators can feed values
- All values are timestamped to ensure data freshness
- Membership changes automatically clean up old operator data
- Gas costs are properly accounted for in EVM operations

## Dependencies

- `frame-support`: Core FRAME support library
- `frame-system`: Core Substrate system library
- `pallet-membership`: (optional) For managing oracle operators
- `pallet-evm`: For EVM precompile support
