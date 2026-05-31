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

## Deploy

```bash
# Testnet
forge script script/Deploy.s.sol \
  --rpc-url rsk_testnet \
  --broadcast \
  --private-key <key>

# Mainnet
forge script script/Deploy.s.sol \
  --rpc-url rsk_mainnet \
  --broadcast \
  --private-key <key>
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
