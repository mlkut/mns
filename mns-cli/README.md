# MNS CLI

CLI for interacting with the MNS (MLKUT Name System) registry on Rootstock testnet.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- An RSK testnet RPC endpoint (default: `https://public-node.testnet.rsk.co`)
- Testnet RBTC to pay for transactions

### Getting testnet RBTC

1. Run `mns init` to generate a key and get your RSK address
2. Visit the [RSK testnet faucet](https://faucet.rootstock.io/faucet/testnet/)
3. Enter your address and request funds
4. Verify with:

```
curl -s https://public-node.testnet.rsk.co \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["<your_address>","latest"],"id":1}'
```

### Getting a testnet RPC API key

The public node is rate-limited. For reliable access, get a free API key from [rpc.rootstock.io](https://rpc.rootstock.io/). Add it to the config file (`~/.config/mns/config.toml`):

```toml
[testnet]
api_key = "your-api-key"
```

With an API key, the CLI will use `https://rpc.rootstock.io/<api_key>/testnet` instead of the public node.

## Commands

### `init`

Initialize a key and config. Generates a secp256k1 private key, stores it in your OS keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager), and prints the derived RSK address.

```
mns init
```

### `register`

Register a new batch of 256 ordinals on the MNS registry.

```
mns register <zsk> <ns>
```

- `zsk` — Zone signing key (32 bytes, 64 hex chars, 0x-prefixed or not)
- `ns` — Namespace string (max 255 bytes)

### `list`

Display batch info and human-readable names for ordinals still using the batch default (omits ordinals moved to independent entries). Read-only — no keychain needed.

```
mns list <ordinal>
```

- `ordinal` — Any ordinal within the batch

### `update`

Create or update a per-ordinal entry.

```
mns update <ordinal> <new_owner> <zsk> <ns>
```

- `ordinal` — Ordinal to update
- `new_owner` — New owner address (0x-prefixed)
- `zsk` — Zone signing key (32 bytes hex)
- `ns` — Namespace string

### `update-batch`

Update batch owner and/or zone config.

```
mns update-batch <ordinal> <new_owner> <zsk> <ns>
```

## Configuration

- Private key is stored in your OS keychain via `keyring`
- Config file: `~/.config/mns/config.toml`
- Network: Rootstock testnet (chain ID 31)
- Contract: `0xe916a48de922e8964542f4c4c66ec4837bbe3445`
- Default RPC: `https://public-node.testnet.rsk.co`
- With API key: `https://rpc.rootstock.io/<api_key>/testnet`

Example config with API keys for both networks:

```toml
[testnet]
api_key = "your-testnet-api-key"

[mainnet]
api_key = "your-mainnet-api-key"
```

## Build

```
cargo build -p mns-cli
```
