# Directory Tree (important paths)

Only paths relevant to the ENZU build/config work. **U** = upstream file (avoid editing;
merge-conflict surface), **E** = ENZU-specific (safe to own).

```
rustdesk/
├── src/                                  U  RustDesk client core (Rust, builds librustdesk)
│   ├── core_main.rs                      U* desktop entry; ENZU hook added at :35
│   ├── flutter_ffi.rs                    U* Flutter FFI; ENZU hook added in initialize()
│   ├── common.rs                         U  get_key(), get_rs_pk(), (load|read)_custom_client()
│   ├── client.rs                         U  outbound connection, key/relay usage
│   ├── rendezvous_mediator.rs            U  hbbs registration, hole-punch/relay negotiation
│   ├── custom_server.rs                  U  upstream exe-name/signed custom client (unused by ENZU)
│   ├── server/                           U  controlled-side (incoming) handling
│   ├── lib.rs                            U* module list; `pub mod enzu_config;` added
│   └── enzu_config.rs                    E  ENZU SOURCE OF TRUTH (servers + public key + policy)
│
├── libs/hbb_common/src/config.rs         U  config store; DEFAULT_SETTINGS, get_option, RS_PUB_KEY
│
├── flutter/
│   ├── lib/                              U  Dart UI (generated_bridge.dart is git-ignored)
│   ├── android/                          U  Android Gradle project
│   │   └── app/
│   │       ├── build.gradle              U  ABI, signing (CI swaps release→debug)
│   │       └── src/main/
│   │           ├── kotlin/**/ffi.kt      U  JNI glue, loads librustdesk.so
│   │           └── jniLibs/arm64-v8a/    -  librustdesk.so + libc++_shared.so (build output)
│   ├── ndk_arm64.sh                      U  cargo-ndk build for aarch64-linux-android
│   └── build_android_deps.sh            U  vcpkg deps for the Android ABI
│
├── build.py                              U  desktop (Windows) build orchestrator
│
├── scripts/
│   └── verify_enzu_config.py             E  build-time verifier (single source of truth check)
│
├── docker/builder/                       E  ENZU Android builder image
│   ├── Dockerfile                        E  pinned toolchain (moved from Dockerfile.enzu-builder)
│   ├── VERSION                           E  "1.0.0" — the image tag
│   ├── README.md                         E  builder image docs
│   ├── build.sh / build.ps1              E  build the image (no publish)
│   └── push.sh / push.ps1                E  optional manual publish (never in CI)
│
├── .github/workflows/enzu-build.yml      E  GitHub: verify-config, generate-bridge, android, windows
├── .gitlab-ci.yml                        E  GitLab: Android arm64 only (builder image)
│
├── docs/
│   ├── ENZU_BUILD_SYSTEM.md              E  CI overview
│   ├── BUILDER_IMAGE.md                  E  redirect → docker/builder/README.md
│   ├── architecture/                     E  ARCHITECTURE/DIRECTORY_TREE/BUILD_FLOW/NETWORK_FLOW/CUSTOMIZATION_GUIDE
│   └── enzu/EMBEDDED_SERVER_CONFIG.md    E  embedded config reference
│
└── .gitignore                            U* + ENZU private-key guards (id_ed25519, **/id_ed25519)
```

`U*` = upstream file with a **minimal** ENZU addition (one call / one line) — kept as small
as possible to minimize future merge conflicts.
