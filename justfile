default:
    @just --list

test:
    cargo test --all-targets

lint:
    cargo +nightly fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
