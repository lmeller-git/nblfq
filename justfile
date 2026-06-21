default:
    @just --list

test:
    cargo test --locked --all-features --all-targets
    cargo test --locked --all-features --doc
    cargo test --locked --no-default-features --all-targets
    cargo test --locked --no-default-features --doc
    cargo +nightly miri test --locked --all-features
    LOOM_MAX_PREEMPTIONS=2 RUSTFLAGS="--cfg loom" cargo test --locked --lib --features dynamic
    RUSTFLAGS="--cfg shuttle" cargo test --locked --lib --features dynamic

lint:
    cargo +nightly fmt --all -- --check
    cargo clippy --all-targets -- -D warnings

check:
    cargo +nightly docs-rs
    cargo hack --feature-powerset check
    cargo semver-checks
