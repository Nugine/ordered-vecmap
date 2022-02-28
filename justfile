dev:
    cargo fmt
    cargo clippy
    cargo test
    cargo +nightly miri test

bench:
    cargo criterion

doc:
    cargo doc --open --no-deps
