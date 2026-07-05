# Auto-rebuild and restart server on file changes
dev:
    cargo watch -x "run -p mns-server"

# Start the server
run:
    cargo run -p mns-server

# Generate Rust bindings from Solidity contracts
bind:
    ./scripts/bind.sh
