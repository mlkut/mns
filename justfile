# Auto-rebuild and restart server on file changes
dev:
    RUST_LOG=mns_server=debug cargo watch -x "run -p mns-server"

# Start the server
run:
    RUST_LOG=mns_server=debug cargo run -p mns-server

# Generate Rust bindings from Solidity contracts
bind:
    ./scripts/bind.sh

# Regenerate name-word candidate pools + feasibility report
curate:
    python3 scripts/curate/generate.py
    python3 scripts/curate/filter.py
    python3 scripts/curate/scan.py --matrix CVCV,CVCC,CVVC,CCVC
    python3 scripts/curate/select.py
    python3 scripts/curate/report.py

# Regenerate the closeness-at-scale analysis (design/name_closeness.md)
closeness:
    python3 scripts/curate/closeness.py

# Name-quality score + tradeoff (needs wordfreq; falls back to offline dict)
quality:
    .venv/bin/python scripts/curate/quality.py || python3 scripts/curate/quality.py --lexicon offline

# As above; also print the quality/overlap pruning tradeoff table
quality-sweep:
    .venv/bin/python scripts/curate/quality.py sweep || python3 scripts/curate/quality.py --lexicon offline sweep

# Measure name-texture + experiment rules (read-only, lists untouched)
texture:
    .venv/bin/python scripts/curate/texture.py report && .venv/bin/python scripts/curate/texture.py experiment

# Verify final lists: no duplicates, well-formed CVCV, no banned/reduplicated tokens
verify:
    python3 scripts/curate/verify.py

# Blind LLM judging pipeline: redraw a 120-name early-ordinal sample, aggregate
# ratings across rounds. (The judging runs as fresh context-free subagents.)
judge-sample:
    python3 scripts/curate/judge.py sample --id 1 --count 120 --cap 1000000000
    python3 scripts/curate/judge.py sample --id 2 --count 120 --cap 1000000000

judge-report:
    .venv/bin/python scripts/curate/judge.py aggregate --ids 1,2

# Print random names from the curated final lists (defaults: 100, first trillion)
# ranges/presets: python3 scripts/curate/sample.py --preset first-million|first-billion|first-trillion|--full|--cap N
sample:
    python3 scripts/curate/sample.py

# Build and deploy to a directory
build dir:
    ./scripts/build.sh {{ dir }}
