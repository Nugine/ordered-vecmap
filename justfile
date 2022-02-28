dev:
    cargo fmt
    cargo clippy
    cargo test
    cargo +nightly miri test

bench:
    cargo criterion
