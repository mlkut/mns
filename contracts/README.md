# MNS Contracts

Rootstock (RSK) smart contracts for the MNS decentralized DNS system.

## Prerequisites

- [Foundry](https://book.getfoundry.sh/getting-started/installation)

## Quick Start

```bash
# Install all dependencies
forge install --no-git foundry-rs/forge-std

# Gnosis Safe test dependencies
forge install --no-git colinnielsen/safe-tools
forge install --no-git safe-global/safe-contracts
forge install --no-git vectorized/solady

# Build & test
forge build
forge test
```

## Environment variables

Copy `.env.dev` to `.env`, populate your address and private key, and once you deploy, populate the contract address.

## Deploy

```bash
# Testnet
forge script script/Deploy.s.sol \
  --rpc-url https://public-node.testnet.rsk.co \
  --broadcast \
  --private-key $privatekey

# Mainnet
forge script script/Deploy.s.sol \
  --rpc-url https://public-node.rsk.co \
  --broadcast \
  --private-key $privatekey
```

## Local Dev

```bash
anvil &                    # local EVM node
forge test                 # tests run against anvil by default
```

## Project Layout

```
src/                       # contract source
test/                      # forge tests (unit + Safe integration)
script/                    # deploy scripts
```

## Verify contract

```bash
forge verify-contract \
  --chain-id 31 \
  --watch \
  --compiler-version v0.8.33 \
  --verifier custom \
  --verifier-url https://be.explorer.testnet.rootstock.io/api/v3/etherscan \
  $contract \
  src/MNSRegistry.sol:MNSRegistry
```
