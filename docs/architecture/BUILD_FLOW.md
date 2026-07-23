# Build Flow

How the ENZU-configured binaries are produced. The ENZU defaults are **compiled into**
`librustdesk` via `src/enzu_config.rs`; no build-time value injection is needed, so
GitHub and GitLab always compile the *same* source-of-truth values.

## Android (arm64-v8a)

```
source (feature/enzu-custom-client)
  └─ scripts/verify_enzu_config.py         # gate: ENZU config valid & single-source
       └─ flutter-rust-bridge codegen       # src/flutter_ffi.rs -> generated_bridge.*
            └─ Rust native build (cargo-ndk, aarch64-linux-android)
                 └─ target/aarch64-linux-android/release/liblibrustdesk.so
                      └─ copy -> flutter/android/app/src/main/jniLibs/arm64-v8a/librustdesk.so
                         copy NDK libc++_shared.so alongside it
                           └─ flutter build apk --release
                                --target-platform android-arm64 --split-per-abi
                                └─ enzu-rustdesk-<VERSION>-android-arm64.apk
```

- **GitHub** (`.github/workflows/enzu-build.yml`): `verify-config` job → `generate-bridge`
  → `android-arm64` (ubuntu-24.04). Artifact `enzu-rustdesk-android-arm64-configured`.
- **GitLab** (`.gitlab-ci.yml`): single `android-arm64` job on
  `enzu/rustdesk-builder:1.0.0`; runs `verify_enzu_config.py`, generates the bridge
  inline (tools baked in the image), then the same build. Same artifact name.
- The ENZU public key/servers live in `librustdesk.so` (compiled Rust constants), applied
  at startup into `DEFAULT_SETTINGS`.

## Windows (x64)

```
source
  └─ scripts/verify_enzu_config.py         # verify-config job (gate)
       └─ flutter-rust-bridge codegen
            └─ python3 build.py --portable --flutter --skip-portable-pack --hwcodec --vram
                 └─ Rust build + Flutter Windows runner (embeds librustdesk)
                      └─ flutter/build/windows/x64/runner/Release -> ./rustdesk/
                           └─ portable artifact  enzu-rustdesk-windows-x64
```

- **GitHub only** (windows-2022). GitLab does not build Windows.
- Same `src/enzu_config.rs` is compiled in, so the Windows client selects the identical
  ENZU servers/key. The Windows `--server` subprocess picks up the defaults via
  `core_main()`.

## Where the config enters the binary

`src/enzu_config.rs` (constants) → compiled into `librustdesk` → `apply_enzu_defaults()`
called at `core_main.rs:35` (desktop, incl. Windows `--server`) and in
`flutter_ffi::initialize()` (Android/Flutter). No CI variable carries the values —
`verify_enzu_config.py` asserts they are present and not duplicated elsewhere.
