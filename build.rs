use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_manifest_dir = Path::new(&cargo_manifest_dir);

    println!("cargo:rerun-if-changed=zig_ast.zig");
    let mut zig = Command::new("zig");
    zig.arg("build");
    zig.arg(match profile.as_str() {
        "release" => "-Doptimize=ReleaseFast",
        "debug" | _ => "-Doptimize=Debug",
    });
    zig.status().unwrap();

    println!("cargo:rerun-if-changed=generate_rust.zig");
    let mut generate = Command::new(cargo_manifest_dir.join("zig-out/bin/generate_rust"));
    generate.arg("src/sys/enums.rs");
    generate.arg("src/sys/full.rs");
    generate.status().unwrap();

    let lib_dir = cargo_manifest_dir.join("zig-out/lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=zig_ast");
}
