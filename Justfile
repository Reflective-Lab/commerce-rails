default:
    @just --list

check:
    cargo check

test:
    cargo test

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
