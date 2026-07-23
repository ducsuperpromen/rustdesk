#[cfg(windows)]
fn build_windows() {
    let file = "src/platform/windows.cc";
    let file2 = "src/platform/windows_delete_test_cert.cc";
    cc::Build::new().file(file).file(file2).compile("windows");
    println!("cargo:rustc-link-lib=WtsApi32");
    println!("cargo:rerun-if-changed={}", file);
    println!("cargo:rerun-if-changed={}", file2);
}

#[cfg(target_os = "macos")]
fn build_mac() {
    let file = "src/platform/macos.mm";
    let mut b = cc::Build::new();
    if let Ok(os_version::OsVersion::MacOS(v)) = os_version::detect() {
        let v = v.version;
        if v.contains("10.14") {
            b.flag("-DNO_InputMonitoringAuthStatus=1");
        }
    }
    b.flag("-std=c++17").file(file).compile("macos");
    println!("cargo:rerun-if-changed={}", file);
}

#[cfg(all(windows, feature = "inline"))]
fn build_manifest() {
    use std::io::Write;
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/icon.ico")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .set_manifest_file("res/manifest.xml");
        match res.compile() {
            Err(e) => {
                write!(std::io::stderr(), "{}", e).unwrap();
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    }
}

fn install_android_deps() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "android" {
        return;
    }
    let mut target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "x86" {
        target_arch = "x86".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    } else {
        target_arch = "arm".to_owned();
    }
    let target = format!("{}-android", target_arch);
    let vcpkg_root = std::env::var("VCPKG_ROOT").unwrap();
    let mut path: std::path::PathBuf = vcpkg_root.into();
    if let Ok(vcpkg_root) = std::env::var("VCPKG_INSTALLED_ROOT") {
        path = vcpkg_root.into();
    } else {
        path.push("installed");
    }
    path.push(target);
    println!(
        "cargo:rustc-link-search={}",
        path.join("lib").to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=ndk_compat");
    println!("cargo:rustc-link-lib=oboe");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=OpenSLES");
}

// ---------------------------------------------------------------------------
// ENZU build-time configuration injection.
//
// Resolves ENZU_ID_SERVER / ENZU_RELAY_SERVER / ENZU_PUBLIC_KEY with precedence
// CI env var  >  repo-root .env (local dev)  >  (absent => src/enzu_config.rs
// fallback constants). A value that is *provided but invalid* fails the build; a
// value that is *completely absent* is left unset so option_env! uses the fallback.
// Uses only std to avoid Cargo.lock churn / new build-dependencies.
// ---------------------------------------------------------------------------
const ENZU_KEYS: [&str; 3] = ["ENZU_ID_SERVER", "ENZU_RELAY_SERVER", "ENZU_PUBLIC_KEY"];
const ENZU_UPSTREAM_RS_PUB_KEY: &str = "OeVuKk5nlHiXp+APNn0Y3pC1Iwpwn44JGqrQCsWqmBw=";
const ENZU_PLACEHOLDERS: [&str; 8] = [
    "<",
    ">",
    "PLACEHOLDER",
    "REPLACE",
    "CHANGEME",
    "TODO",
    "XXXX",
    "EXAMPLE",
];

fn enzu_read_dotenv() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&dir).join(".env");
    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let mut v = v.trim();
                if v.len() >= 2
                    && ((v.starts_with('"') && v.ends_with('"'))
                        || (v.starts_with('\'') && v.ends_with('\'')))
                {
                    v = &v[1..v.len() - 1];
                }
                map.insert(k.trim().to_string(), v.to_string());
            }
        }
    }
    map
}

// Length in bytes if `s` is valid standard base64, else None.
fn enzu_b64_decoded_len(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.is_empty() || s.len() % 4 != 0 {
        return None;
    }
    let bytes = s.as_bytes();
    let pad = bytes.iter().rev().take_while(|&&c| c == b'=').count();
    if pad > 2 {
        return None;
    }
    for &c in &bytes[..bytes.len() - pad] {
        if !matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/') {
            return None;
        }
    }
    Some(s.len() / 4 * 3 - pad)
}

fn enzu_validate(key: &str, value: &str) {
    if value.is_empty() {
        panic!(
            "ENZU build config: {key} is provided but empty. Set a valid value, or \
             leave it unset (remove from env/.env) to use the built-in ENZU default."
        );
    }
    let upper = value.to_uppercase();
    for marker in ENZU_PLACEHOLDERS {
        if upper.contains(marker) {
            panic!("ENZU build config: {key} contains placeholder text ({marker:?}): {value:?}");
        }
    }
    if key == "ENZU_PUBLIC_KEY" {
        if value.contains(char::is_whitespace) {
            panic!("ENZU build config: ENZU_PUBLIC_KEY must be a single whitespace-free line");
        }
        if value == ENZU_UPSTREAM_RS_PUB_KEY {
            panic!("ENZU build config: ENZU_PUBLIC_KEY equals the public RustDesk key, not ENZU");
        }
        match enzu_b64_decoded_len(value) {
            Some(32) => {}
            Some(n) => {
                panic!("ENZU build config: ENZU_PUBLIC_KEY decodes to {n} bytes, expected 32")
            }
            None => panic!("ENZU build config: ENZU_PUBLIC_KEY is not valid base64"),
        }
    } else {
        // hostname (ID / relay server)
        let low = value.to_lowercase();
        for bad in ["localhost", "127.0.0.1", "0.0.0.0", "rustdesk.com"] {
            if low.contains(bad) {
                panic!("ENZU build config: {key} points at forbidden/public infra ({bad:?}): {value:?}");
            }
        }
    }
}

fn set_enzu_build_config() {
    let dotenv = enzu_read_dotenv();
    for key in ENZU_KEYS {
        println!("cargo:rerun-if-env-changed={key}");
        // Precedence: CI/process env first, then local .env. `std::env::var` returns
        // Ok("") for a present-but-empty var (e.g. an unset GitHub Variable mapped
        // into env) -> treated as "provided" -> validated -> fails on empty.
        let provided = match std::env::var(key) {
            Ok(v) => Some(v),
            Err(_) => dotenv.get(key).cloned(),
        };
        if let Some(value) = provided {
            let value = value.trim();
            enzu_validate(key, value);
            println!("cargo:rustc-env={key}={value}");
        }
        // Absent everywhere: leave unset so option_env! falls back to the constant.
    }
    println!("cargo:rerun-if-changed=.env");
}

fn main() {
    hbb_common::gen_version();
    set_enzu_build_config();
    install_android_deps();
    #[cfg(all(windows, feature = "inline"))]
    build_manifest();
    #[cfg(windows)]
    build_windows();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        #[cfg(target_os = "macos")]
        build_mac();
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
