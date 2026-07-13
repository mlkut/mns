# Auto-rebuild and restart server on file changes
dev:
    RUST_LOG=mns_server=debug cargo watch -x "run -p mns-server"

# Start the server
run:
    RUST_LOG=mns_server=debug cargo run -p mns-server

# Generate Rust bindings from Solidity contracts
bind:
    ./scripts/bind.sh

# Build the server
build:
    ./scripts/build.sh
