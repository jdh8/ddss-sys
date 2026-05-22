use anyhow::Context as _;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").context("OUT_DIR not set")?;
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?,
    );

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

    // GCC/clang-spelled mirror of the cc::Build flags above, for clangd's
    // compile_commands.json. Keep in sync with the cc::Build call.
    let mut flags: Vec<String> = vec![
        "-xc++".into(),
        "-std=c++17".into(),
        "-Ivendor/include".into(),
        "-Ivendor/src".into(),
    ];
    for d in &defines {
        flags.push(format!("-D{d}"));
    }
    if !is_packaging(&manifest_dir) {
        write_compile_commands(&manifest_dir, &sources, &flags)?;
    }
    Ok(())
}

/// `cargo publish` extracts the crate into `<workspace>/target/package/<name>-<version>/`
/// and forbids the build script from touching anything outside `OUT_DIR`. Detect that
/// layout so we skip the clangd-only `compile_commands.json` write in that case.
fn is_packaging(manifest_dir: &Path) -> bool {
    let mut comps = manifest_dir.components().rev();
    comps.next().is_some()
        && comps.next().map(std::path::Component::as_os_str) == Some("package".as_ref())
        && comps.next().map(std::path::Component::as_os_str) == Some("target".as_ref())
}

fn write_compile_commands(
    manifest_dir: &Path,
    sources: &[PathBuf],
    flags: &[String],
) -> anyhow::Result<()> {
    let dir = manifest_dir.display().to_string();
    let entries: Vec<String> = sources
        .iter()
        .map(|p| {
            let file = p.display().to_string();
            let args = std::iter::once("clang++".to_string())
                .chain(flags.iter().cloned())
                .chain(["-c".into(), file.clone()])
                .map(|s| json_string(&s))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "  {{ \"directory\": {}, \"file\": {}, \"arguments\": [{}] }}",
                json_string(&dir),
                json_string(&file),
                args,
            )
        })
        .collect();
    std::fs::write(
        manifest_dir.join("compile_commands.json"),
        format!("[\n{}\n]\n", entries.join(",\n")),
    )
    .context("writing compile_commands.json")?;
    Ok(())
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
