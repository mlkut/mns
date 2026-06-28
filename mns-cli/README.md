# MNS CLI

CLI for interacting with the MNS (MLKUT Name System) registry on Rootstock testnet.

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

Display batch info and all 256 ordinals with their human-readable names.

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
- RPC: `https://public-node.testnet.rsk.co`

## Build

```
cargo build -p mns-cli
```
