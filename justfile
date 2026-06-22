default:
    @just --list

test-rs:
    cargo test --locked --all-features --all-targets
    cargo test --locked --all-features --doc
    cargo test --locked --no-default-features --all-targets
    cargo test --locked --no-default-features --doc
    cargo +nightly miri test --locked --all-features
    LOOM_MAX_PREEMPTIONS=2 RUSTFLAGS="--cfg loom" cargo test --locked --lib --features dynamic
    RUSTFLAGS="--cfg shuttle" cargo test --locked --lib --features dynamic

lint-rs:
    cargo +nightly fmt --all -- --check
    cargo clippy --all-targets -- -D warnings

check-rs:
    cargo +nightly docs-rs
    cargo hack --feature-powerset check
    cargo semver-checks

build-py:
    uv sync
    uv run maturin develop

test-py: build-py
    uv run pytest

lint-py:
    uv run ruff check python
    uv run ruff format
    uv run ruff format --check python
    uv run pyright


test: test-rs test-py

lint: lint-rs lint-py

check: check-rs
