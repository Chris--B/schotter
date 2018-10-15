@echo off

cls
cargo check                        || goto :EOF
cargo test                         || goto :EOF
cargo doc --document-private-items || goto :EOF
cargo build && cargo build --release
cargo run
