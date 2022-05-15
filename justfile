dev:
    cargo fmt
    cargo clippy
    cargo miri test
    cargo test

bench:
    cargo criterion

doc:
    cargo doc --open --no-deps
