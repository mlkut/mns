# mns-server

MNS DNS server — syncs contract state via event logs and serves DNS over HTTPS.

## Prerequisites

- Rust toolchain
- Rootstock RPC access (event polling requires an API key)

## Quick start

```bash
# Create .env with your API key (see below)
echo 'RSK_RPC_API_KEY=your_key_here' > .env

# Run with auto-rebuild on file changes
just dev

# Or run once
just run
```

## Configuration

All options are available via CLI flags:

| Flag | Default | Description |
|------|---------|-------------|
| `--rpc-api-key` | — | Rootstock RPC API key (env: `RSK_RPC_API_KEY`) |
| `--rpc-url` | `https://public-node.testnet.rsk.co` | Rootstock RPC URL |
| `--network` | `testnet` | Network name |
| `--listen` | `127.0.0.1:3000` | HTTP listen address |
| `--max-block-range` | `2000` | Max blocks per `eth_getLogs` query |
| `--db-path` | `.mns-server/db` | LMDB database directory |
| `--poll-interval` | `12` | Event poll interval in seconds |

### `.env` file

Create a `.env` file (already gitignored) in the project root using
standard dotenv format (no `export` keyword):

```env
RSK_RPC_API_KEY=your_key_here
```

The server loads it automatically at startup via `dotenvy`.

### API key and RPC URL

Event polling requires an API key. The public testnet node
(`public-node.testnet.rsk.co`) does not support `eth_getLogs`.

When the key is set and `--rpc-url` is the default, the URL becomes
`https://public-node.testnet.rsk.co/{api_key}`, which won't work.
Override `--rpc-url` to match your provider's format:

```bash
RSK_RPC_API_KEY=your_key_here just dev -- --rpc-url https://rpc.rootstock.io/
```
