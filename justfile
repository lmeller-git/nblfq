default:
    @just --list

test:
    cargo test --locked --all-features --all-targets
    cargo test --locked --all-features --doc
    cargo test --locked --no-default-features --all-targets
    cargo test --locked --no-default-features --doc
    cargo miri test --locked --all-features
    LOOM_MAX_PREEMPTIONS=2 RUSTFLAGS="--cfg loom" cargo test --release --locked --lib
    RUSTFLAGS="--cfg shuttle" cargo test --release --locked --lib

lint:
    cargo +nightly fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
