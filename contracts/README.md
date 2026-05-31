# MNS Contracts

Rootstock (RSK) smart contracts for the MNS decentralized DNS system.

## Quick Start

```bash
# Install dependencies (gitignored — not part of the repo)
forge install --no-git foundry-rs/forge-std

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
test/                      # forge tests
script/                    # deploy scripts
```

Dependencies live in `lib/` (gitignored). After cloning, run `forge install --no-git foundry-rs/forge-std` to restore them.
