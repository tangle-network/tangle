# Tangle Network User Simulation

This project simulates multiple users interacting with the Tangle Network using TypeScript and Polkadot.js.

## Prerequisites

- Node.js (v16 or higher)
- npm
- A running Tangle Network node (local or remote)

## Setup

1. Install dependencies:
```bash
npm install
```

2. Build the project:
```bash
npm run build
```

3. Run the simulation:
```bash
npm start
```

## Development

For development with hot-reloading:
```bash
npm run dev
```

## Project Structure

- `src/index.ts`: Main entry point and simulation logic
- `dist/`: Compiled JavaScript files
- `tsconfig.json`: TypeScript configuration
- `package.json`: Project dependencies and scripts

## Adding More Simulations

Add new simulation methods to the UserSimulation class in `src/index.ts`. Each method can represent different user interactions with the Tangle Network.
