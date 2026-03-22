How it works:

1. C API is defined in `zig_ast.zig`
2. `generate_rust.zig` is used to generate `src/sys/enums.rs` and `src/sys/full.rs`
3. Low-level Rust API is defined in  `src/sys/mod.rs`
4. Higher-level Rust API is defined in `src/lib.rs`
5. Test program `src/main.rs` uses the higher-level API

Built with Zig 0.15.2 and Rust 1.94.0
