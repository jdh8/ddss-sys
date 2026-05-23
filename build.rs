use anyhow::Context as _;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").context("OUT_DIR not set")?;

    bindgen::Builder::default()
        .header("src/wrapper.h")
        .use_core()
        .allowlist_file("vendor/.*")
        .clang_arg("-xc++")
        .clang_arg("-Ivendor/include")
        .prepend_enum_name(false)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?
        .write_to_file(PathBuf::from(out_dir).join("bindings.rs"))?;

    let sources: Vec<PathBuf> = glob::glob("vendor/src/*.cpp")?.flatten().collect();

    let mut defines: Vec<&'static str> = vec!["DDS_THREADS_STL"];

    if std::env::var_os("CARGO_FEATURE_DEBUG_DUMP").is_none() {
        defines.push("DDS_NO_DUMP_ON_ERROR");
    }

    for (feat, macro_name) in [
        ("CARGO_FEATURE_DEBUG_TOP_LEVEL", "DDS_TOP_LEVEL"),
        ("CARGO_FEATURE_DEBUG_AB_STATS", "DDS_AB_STATS"),
        ("CARGO_FEATURE_DEBUG_TT_STATS", "DDS_TT_STATS"),
        ("CARGO_FEATURE_DEBUG_TIMING", "DDS_TIMING"),
        ("CARGO_FEATURE_DEBUG_MOVES", "DDS_MOVES"),
    ] {
        if std::env::var_os(feat).is_some() {
            defines.push(macro_name);
        }
    }

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .files(&sources)
        .include("vendor/include")
        .include("vendor/src")
        .std("c++17")
        .cargo_warnings(false);
    for d in &defines {
        build.define(d, None);
    }
    if cfg!(target_env = "msvc") {
        build.define("_CRT_SECURE_NO_WARNINGS", None);
    }
    build.try_compile("dds")?;
    Ok(())
}
