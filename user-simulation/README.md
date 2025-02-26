# Tangle Network User Simulation

This package contains user simulation tests for the Tangle Network. It helps verify the functionality of the network by simulating real user interactions.

## Prerequisites

- Node.js (v16 or higher)
- A running Tangle Network node (locally at ws://localhost:9944)
- yarn or npm installed

## Setup

1. Install dependencies:
```bash
yarn install
```

2. Build the TypeScript code:
```bash
yarn build
```

## Running the Simulation

To run the user simulation:

```bash
yarn start
```

This will:
1. Connect to your local Tangle node
2. Create test accounts
3. Simulate user interactions with the network
4. Output results of the simulation

## Development

For development with hot-reloading:
```bash
yarn dev
```

## Running Tests

To run the test suite:
```bash
yarn test
```
