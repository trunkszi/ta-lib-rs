//! Build script for statically compiling upstream `TA-Lib` via CMake and generating
//! the complete Rust FFI bindings from `ABI.manifest`.

use std::{
    collections::BTreeSet,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

const REPOSITORY: &str = "https://github.com/TA-Lib/ta-lib.git";
const DEFAULT_BRANCH: &str = "main";
const DEFAULT_TAG: &str = "v0.8.1";

const EXPECTED_MIN_FUNCTIONS: usize = if cfg!(feature = "release-tag") {
    2_451
} else {
    3_122
};
const EXPECTED_MIN_STREAMS: usize = if cfg!(feature = "release-tag") {
    201
} else {
    237
};

fn is_source_tree(path: &Path) -> bool {
    path.join("CMakeLists.txt").is_file()
        && path.join("ABI.manifest").is_file()
        && path.join("include/ta_libc.h").is_file()
}

fn target_root(manifest: &Path, out_dir: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                manifest.join("../..").join(path)
            }
        })
        .or_else(|| {
            out_dir
                .ancestors()
                .find(|path| path.file_name().is_some_and(|name| name == "build"))
                .and_then(Path::parent)
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| out_dir.to_path_buf())
}

fn parse_semver_tag(tag: &str) -> Option<(u32, u32, u32)> {
    let rest = tag.strip_prefix('v')?;
    if rest.contains('-') {
        return None;
    }
    let mut parts = rest.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn resolve_latest_release_tag(target_dir: &Path) -> String {
    if let Ok(entries) = fs::read_dir(target_dir) {
        let mut cached: Vec<(u32, u32, u32, String)> = entries
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if !is_source_tree(&path) {
                    return None;
                }
                let name = entry.file_name().into_string().ok()?;
                let tag = name.strip_prefix("ta-lib-")?;
                let (maj, min, pat) = parse_semver_tag(tag)?;
                Some((maj, min, pat, tag.to_owned()))
            })
            .collect();
        cached.sort();
        if let Some((_, _, _, tag)) = cached.pop() {
            return tag;
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", REPOSITORY, "v*"])
        .output()
        && output.status.success()
        && let Ok(stdout) = String::from_utf8(output.stdout)
    {
        let mut tags: Vec<(u32, u32, u32, String)> = stdout
            .lines()
            .filter_map(|line| {
                let ref_name = line.split_whitespace().nth(1)?;
                let tag = ref_name.strip_prefix("refs/tags/")?;
                let (maj, min, pat) = parse_semver_tag(tag)?;
                Some((maj, min, pat, tag.to_owned()))
            })
            .collect();
        tags.sort();
        if let Some((_, _, _, tag)) = tags.pop() {
            return tag;
        }
    }

    DEFAULT_TAG.to_owned()
}

fn verify_cached_tag(source: &Path, expected_tag: &str) {
    if source.join(".git").exists() {
        let output = Command::new("git")
            .arg("-C")
            .arg(source)
            .args(["describe", "--tags", "--exact-match", "HEAD"])
            .output()
            .expect("verify TA-Lib tag");
        assert!(
            output.status.success(),
            "failed to describe TA-Lib tag at {}",
            source.display()
        );
        let actual = String::from_utf8(output.stdout)
            .expect("UTF-8 tag")
            .trim()
            .to_owned();
        assert_eq!(
            actual,
            expected_tag,
            "TA-Lib cached tag mismatch at {}",
            source.display()
        );
    }
}

fn verify_cached_branch(source: &Path, expected_branch: &str) {
    if source.join(".git").exists() {
        let output = Command::new("git")
            .arg("-C")
            .arg(source)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .expect("verify TA-Lib branch");
        assert!(
            output.status.success(),
            "failed to verify TA-Lib branch at {}",
            source.display()
        );
        let actual = String::from_utf8(output.stdout)
            .expect("UTF-8 branch")
            .trim()
            .to_owned();
        assert_eq!(
            actual,
            expected_branch,
            "TA-Lib cached branch mismatch at {}",
            source.display()
        );
    }
}

fn clone_into_target(target_dir: &Path, git_ref: &str, verify_tag: bool) -> PathBuf {
    let cache = target_dir.join(format!("ta-lib-{git_ref}"));
    if is_source_tree(&cache) {
        if verify_tag {
            verify_cached_tag(&cache, git_ref);
        } else {
            verify_cached_branch(&cache, git_ref);
        }
        return cache;
    }

    let staging = target_dir.join(format!("ta-lib-{git_ref}.fetch-{}", std::process::id()));
    drop(fs::remove_dir_all(&staging));
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", git_ref, REPOSITORY])
        .arg(&staging)
        .status()
        .expect("run git clone for TA-Lib");
    assert!(
        status.success(),
        "failed to clone TA-Lib ref {git_ref} from {REPOSITORY}"
    );
    if verify_tag {
        verify_cached_tag(&staging, git_ref);
    } else {
        verify_cached_branch(&staging, git_ref);
    }
    if let Err(err) = fs::rename(&staging, &cache) {
        assert!(
            is_source_tree(&cache),
            "failed to move staged TA-Lib clone into {}: {err}",
            cache.display()
        );
        drop(fs::remove_dir_all(&staging));
    }
    if verify_tag {
        verify_cached_tag(&cache, git_ref);
    } else {
        verify_cached_branch(&cache, git_ref);
    }
    cache
}

fn resolve_source(manifest: &Path, out_dir: &Path) -> PathBuf {
    if let Ok(configured) = env::var("TA_LIB_DIR") {
        let trimmed = configured.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if is_source_tree(&candidate) {
                return candidate;
            }
            panic!(
                "TA_LIB_DIR={} does not point to a valid TA-Lib CMake source tree",
                candidate.display()
            );
        }
    }

    let target_dir = target_root(manifest, out_dir);
    fs::create_dir_all(&target_dir).expect("create target directory");
    if cfg!(feature = "release-tag") {
        let tag = resolve_latest_release_tag(&target_dir);
        clone_into_target(&target_dir, &tag, true)
    } else {
        clone_into_target(&target_dir, DEFAULT_BRANCH, false)
    }
}

fn map_c_type(c_ty: &str, is_return: bool) -> String {
    let ty = c_ty.trim();
    if ty == "void" {
        return if is_return {
            "()".to_owned()
        } else {
            "core::ffi::c_void".to_owned()
        };
    }
    match ty {
        "int" => "core::ffi::c_int".to_owned(),
        "unsigned int" => "core::ffi::c_uint".to_owned(),
        "double" => "f64".to_owned(),
        "float" => "f32".to_owned(),
        "const char *" => "*const core::ffi::c_char".to_owned(),
        "const char **" => "*mut *const core::ffi::c_char".to_owned(),
        "const double *" => "*const f64".to_owned(),
        "double *" => "*mut f64".to_owned(),
        "const float *" => "*const f32".to_owned(),
        "float *" => "*mut f32".to_owned(),
        "int *" => "*mut core::ffi::c_int".to_owned(),
        "const int *" => "*const core::ffi::c_int".to_owned(),
        "void *" => "*mut core::ffi::c_void".to_owned(),
        "const void *" => "*const core::ffi::c_void".to_owned(),
        _ => {
            if let Some(base) = ty
                .strip_prefix("const ")
                .and_then(|s| s.strip_suffix(" **"))
            {
                format!("*mut *const {}", base.trim())
            } else if let Some(base) = ty.strip_prefix("const ").and_then(|s| s.strip_suffix(" *"))
            {
                format!("*const {}", base.trim())
            } else if let Some(base) = ty.strip_suffix(" **") {
                format!("*mut *mut {}", base.trim())
            } else if let Some(base) = ty.strip_suffix(" *") {
                format!("*mut {}", base.trim())
            } else {
                ty.to_owned()
            }
        }
    }
}

fn generate_bindings(source: &Path, out_dir: &Path) {
    let manifest_path = source.join("ABI.manifest");
    let reader = BufReader::new(File::open(&manifest_path).expect("open ABI.manifest"));

    let mut stream_types = BTreeSet::new();
    let mut functions = Vec::new();

    for line in reader.lines() {
        let line = line.expect("read line from ABI.manifest");
        let trimmed = line.trim();
        if !trimmed.starts_with("function ") {
            continue;
        }
        let rest = trimmed.strip_prefix("function ").expect("function prefix");
        let open_paren = rest.find('(').expect("open paren in function line");
        let close_paren = rest.rfind(')').expect("close paren in function line");
        let header = rest[..open_paren].trim();
        let params_str = rest[open_paren + 1..close_paren].trim();

        let (ret_c, fn_name) = if let Some(idx) = header.rfind(" *TA_") {
            (format!("{} *", header[..idx].trim()), &header[idx + 2..])
        } else {
            let idx = header.rfind(" TA_").expect("TA_ function name");
            (header[..idx].trim().to_owned(), &header[idx + 1..])
        };

        let mut rust_params = Vec::new();
        if params_str != "void" && !params_str.is_empty() {
            for (i, raw_param) in params_str.split(',').enumerate() {
                let p = raw_param.trim();
                for token in p.split_whitespace() {
                    let clean = token.trim_matches('*');
                    if clean.starts_with("TA_") && clean.ends_with("_Stream") {
                        stream_types.insert(clean.to_owned());
                    }
                }
                let rust_ty = map_c_type(p, false);
                rust_params.push(format!("arg{i}: {rust_ty}"));
            }
        }
        let ret_rust = map_c_type(&ret_c, true);
        let sig = if ret_rust == "()" {
            format!("    pub fn {fn_name}({});", rust_params.join(", "))
        } else {
            format!(
                "    pub fn {fn_name}({}) -> {ret_rust};",
                rust_params.join(", ")
            )
        };
        functions.push(sig);
    }

    assert!(
        functions.len() >= EXPECTED_MIN_FUNCTIONS,
        "unsupported TA-Lib ABI function count: {} (expected >= {EXPECTED_MIN_FUNCTIONS})",
        functions.len()
    );
    assert!(
        stream_types.len() >= EXPECTED_MIN_STREAMS,
        "unsupported TA-Lib ABI stream count: {} (expected >= {EXPECTED_MIN_STREAMS})",
        stream_types.len()
    );

    let mut out = File::create(out_dir.join("bindings.rs")).expect("create bindings.rs");
    writeln!(
        out,
        "// Auto-generated from upstream TA-Lib ABI.manifest ({} functions).",
        functions.len()
    )
    .expect("write header");

    for st in &stream_types {
        writeln!(
            out,
            "#[repr(C)]\n#[derive(Debug)]\npub struct {st} {{\n    _private: [u8; 0],\n}}"
        )
        .expect("write stream struct");
    }

    writeln!(out, "\nunsafe extern \"C\" {{").expect("write extern block start");
    for f in &functions {
        writeln!(out, "{f}").expect("write function decl");
    }
    writeln!(out, "}}").expect("write extern block end");
    writeln!(
        out,
        "\npub const GENERATED_FUNCTION_COUNT: usize = {};",
        functions.len()
    )
    .expect("write count");
    writeln!(
        out,
        "pub const GENERATED_STREAM_TYPE_COUNT: usize = {};",
        stream_types.len()
    )
    .expect("write stream count");
}

fn main() {
    println!("cargo:rerun-if-env-changed=TA_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let source = resolve_source(&manifest, &out_dir);
    println!(
        "cargo:rerun-if-changed={}",
        source.join("ABI.manifest").display()
    );
    generate_bindings(&source, &out_dir);

    let install_dir = cmake::Config::new(&source)
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_STATIC_LIBS", "ON")
        .define("BUILD_DEV_TOOLS", "OFF")
        .define("BUILD_BENCHMARKS", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .profile("Release")
        .build();

    for subdir in ["lib", "lib64", "build"] {
        let candidate = install_dir.join(subdir);
        if candidate.is_dir() {
            println!("cargo:rustc-link-search=native={}", candidate.display());
        }
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        if install_dir.join("lib/ta-lib-static.lib").is_file() {
            println!("cargo:rustc-link-lib=static=ta-lib-static");
        } else {
            println!("cargo:rustc-link-lib=static=ta-lib");
        }
    } else {
        if install_dir.join("lib/libta-lib.a").is_file()
            || install_dir.join("lib64/libta-lib.a").is_file()
        {
            println!("cargo:rustc-link-lib=static=ta-lib");
        } else if install_dir.join("lib/libta-lib-static.a").is_file() {
            println!("cargo:rustc-link-lib=static=ta-lib-static");
        } else {
            println!("cargo:rustc-link-lib=static=ta-lib");
        }
        println!("cargo:rustc-link-lib=dylib=m");
    }
}
