# Auto-rebuild and restart server on file changes
dev:
    RUST_LOG=mns_server=debug cargo watch -x "run -p mns-server"

# Start the server
run:
    RUST_LOG=mns_server=debug cargo run -p mns-server

# Generate Rust bindings from Solidity contracts
bind:
    ./scripts/bind.sh

# Run the name tests (encoding, permutation, mns::luts pools + LUTs, goldens)
test-names:
    cargo test -p mns name::

# Build and deploy to a directory
build dir:
    ./scripts/build.sh {{ dir }}