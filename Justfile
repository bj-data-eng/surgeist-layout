# List available recipes.
default:
    @just --list

# Format Rust sources.
fmt:
    @./scripts/run-cargo-task.sh fmt

# Check Rust formatting without changing files.
fmt-check:
    @./scripts/run-cargo-task.sh fmt-check

# Check the default crate configuration.
check:
    @./scripts/run-cargo-task.sh check

# Test the default crate configuration.
test:
    @./scripts/run-cargo-task.sh test

# Lint the default crate configuration.
clippy:
    @./scripts/run-cargo-task.sh clippy

# Run all default verification checks.
verify:
    @./scripts/run-verification.sh default

# Check the generator feature configuration.
generator-check:
    @./scripts/run-cargo-task.sh generator-check

# Test the generator feature configuration.
generator-test:
    @./scripts/run-cargo-task.sh generator-test

# Lint the generator feature configuration.
generator-clippy:
    @./scripts/run-cargo-task.sh generator-clippy

# Run all generator verification checks.
verify-generator:
    @./scripts/run-verification.sh generator

# Run the full checked-in browser-parity corpus.
parity-all:
    @./scripts/run-browser-parity-task.sh parity-all

# Validate the checked-in browser-parity corpus.
corpus-check:
    @./scripts/run-browser-parity-task.sh corpus-check

# Validate the checked-in Taffy parity corpus.
taffy-check:
    @./scripts/run-browser-parity-task.sh taffy-check
