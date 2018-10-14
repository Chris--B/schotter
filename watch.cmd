@echo off

cls
cargo test                         || goto :EOF
cargo doc --document-private-items || goto :EOF
cargo run --
