# Source Layout — Current Physical Layout vs. ENZU Conceptual Architecture

This document is the Phase 2 deliverable of the ENZU source-structure migration
(baseline `84636ba6a`, safety tag `enzu-pre-structure-migration`). It maps the
**current, physical** RustDesk tree to the **target, conceptual** ENZU Remote
architecture, and records why each path is or isn't safe to move today.

It complements, and does not repeat, the file-level detail already in:
- [ARCHITECTURE.md](ARCHITECTURE.md) — Rust/Flutter/bridge/Android/Windows/config data flow.
- [DIRECTORY_TREE.md](DIRECTORY_TREE.md) — per-file ownership (`U` upstream / `E` ENZU) with line refs.
- [BUILD_FLOW.md](BUILD_FLOW.md), [NETWORK_FLOW.md](NETWORK_FLOW.md), [CUSTOMIZATION_GUIDE.md](CUSTOMIZATION_GUIDE.md).
- [CI.md](CI.md) — GitHub workflow audit (Phase 6).
- [UPSTREAM_SYNC.md](UPSTREAM_SYNC.md) — future upstream-update procedure (Phase 7).

## 1. Current physical layout (top level)

```
rustdesk/
├── Cargo.toml / Cargo.lock / build.rs   Rust workspace root (must stay at repo root)
├── build.py                             Windows/desktop build orchestrator
├── vcpkg.json                           native dep manifest (path checked by flutter/build_android_deps.sh)
├── .env / .env.example                  ENZU local dev config (git-ignored .env)
├── .cargo/config.toml                   rustflags per target triple
├── src/                                 Rust client core -> crate `librustdesk`
├── libs/                                Rust workspace members (scrap, enigo, clipboard, virtual_display,
│                                        remote_printer, portable, libxdo-sys-stub, hbb_common)
│   └── hbb_common/                      GIT SUBMODULE -> github.com/rustdesk/hbb_common (see §8)
├── flutter/                             Flutter project root (UI + all platform runners)
│   ├── lib/                             shared Dart UI
│   ├── android/                         Android Gradle project (must stay 2 levels under flutter/)
│   ├── windows/                         Windows CMake/Runner project
│   ├── macos/, linux/, ios/             other upstream platform runners (out of ENZU scope)
│   ├── ndk_arm64.sh, build_android_deps.sh, build_fdroid.sh, ...   platform build helpers
├── scripts/verify_enzu_config.py        ENZU config verifier (hardcoded path to src/enzu_config.rs)
├── docker/builder/                      ENZU Android builder image (GitLab CI)
├── docs/                                upstream docs + ENZU docs (architecture/, enzu/)
├── .github/workflows/                   GitHub Actions (see CI.md)
└── .gitlab-ci.yml                       GitLab CI (Android-only, additive, ENZU-owned)
```

## 2. Target conceptual architecture

```
ENZU Remote
├── core/       Rust shared functionality: rendezvous, relay, remote desktop, codec, input, encryption
├── android/    ENZU Android-specific integration
├── windows/    ENZU Windows-specific integration
├── flutter/    shared Flutter UI
├── config/     ENZU configuration
├── scripts/    Android/Windows build tooling
├── docs/architecture/
└── .github/workflows/enzu-build.yml
```

This is a **conceptual ownership map**, not a physical directory mandate. Per the
migration's safety rule, the physical tree is *not* forced into this shape this phase.

## 3. Current directories implementing each conceptual subsystem

| Conceptual owner | Current physical location(s) |
|---|---|
| `core/` — rendezvous | `src/rendezvous_mediator.rs`, `libs/hbb_common/src/config.rs` |
| `core/` — relay | `src/client.rs` (relay usage), `libs/hbb_common` (relay proto) |
| `core/` — remote desktop | `src/server/**`, `src/client.rs`, `src/ui_session_interface.rs` |
| `core/` — codec/video | `libs/scrap/` |
| `core/` — input | `libs/enigo/`, `src/keyboard.rs`, `src/clipboard*.rs` |
| `core/` — encryption | `src/common.rs` (`get_key`, `get_rs_pk`), RustDesk's own e2e crypto in `libs/hbb_common` |
| `android/` | `flutter/android/**` (Gradle/Kotlin/JNI), `flutter/ndk_*.sh`, `flutter/build_android_deps.sh` |
| `windows/` | `flutter/windows/**` (CMake/Runner), `build.py`, `.cargo/config.toml` (msvc rustflags) |
| `flutter/` (shared UI) | `flutter/lib/**` |
| `config/` | `src/enzu_config.rs`, `build.rs` (`set_enzu_build_config`), `.env(.example)`, `scripts/verify_enzu_config.py`, `docs/enzu/EMBEDDED_SERVER_CONFIG.md` |
| `scripts/` | `scripts/verify_enzu_config.py`, `docker/builder/*`, `flutter/build_*.sh`, `flutter/ndk_*.sh` |

Ownership marks (`U` upstream / `E` ENZU / `U*` upstream-with-minimal-ENZU-hook) are
already tracked per file in [DIRECTORY_TREE.md](DIRECTORY_TREE.md) — that table is the
source of truth for "is this line safe to touch during upstream merges."

## 4. Shared vs. platform-specific components

**Shared by Android and Windows** (compiled once as `librustdesk`, consumed by both):
- All of `src/**` and the Rust workspace members in `libs/**` (except `libs/virtual_display`,
  which is Windows-only per `Cargo.toml`'s `cfg(target_os = "windows")` dependency block).
- `flutter/lib/**` (Dart UI) — same code, different platform runner underneath.
- `src/enzu_config.rs` / `build.rs` ENZU injection — identical on every platform.
- The generated bridge (`flutter_rust_bridge_codegen` output) — one `generate-bridge` CI
  job feeds both the `android-arm64` and `windows-x64` jobs in `enzu-build.yml`.

**Platform-specific:**
- Android: `flutter/android/**` (Gradle, Kotlin JNI glue, `jniLibs/` native lib placement),
  `cargo-ndk` cross-compilation (`flutter/ndk_arm64.sh`), `vcpkg` Android triplet
  (`flutter/build_android_deps.sh`).
- Windows: `flutter/windows/**` (CMake `Runner`), `build.py` orchestration, MSVC static-CRT
  `rustflags` (`.cargo/config.toml`), `vcpkg` `x64-windows-static` triplet, the custom
  RustDesk Flutter engine download step in `enzu-build.yml`.

## 5. Paths that cannot safely be moved yet, and why

Every path below has at least one **relative-path dependency** discovered during the
Phase 1 audit. Moving it without updating every dependent would break the build; each
required change is invasive enough that the migration's safety rule ("preserve a
buildable repository") rules it out for this phase.

| Path | Why it can't move yet |
|---|---|
| `Cargo.toml`, `Cargo.lock`, `build.rs` | Cargo requires `build.rs` at the package root unless `Cargo.toml` is restructured; workspace member paths (`libs/scrap`, `libs/hbb_common`, ...) are relative to this root. |
| `src/` | It's the crate root referenced by `Cargo.toml` (`[lib]`, `[[bin]]` paths) and by `build.rs` (`src/platform/windows.cc`, `src/platform/macos.mm`). |
| `libs/hbb_common` | **Git submodule** pointing at `github.com/rustdesk/hbb_common` (see `.gitmodules`), not a plain directory — moving it means re-pointing the submodule and every relative reference into it (see below), and decouples it from upstream's own submodule path expectations. |
| `libs/*` (all workspace members) | Listed by relative path in `Cargo.toml`'s `[workspace] members`; `libs/scrap` is also referenced with a `path =` dependency. |
| `vcpkg.json` | `flutter/build_android_deps.sh` hard-checks `"${SCRIPTDIR}/../vcpkg.json"` — i.e. exactly one level above `flutter/`. |
| `flutter/` | `flutter/windows/CMakeLists.txt:106` hardcodes `../../target/<profile>/librustdesk.dll` — two levels up from `flutter/windows/`, i.e. the Cargo `target/` at the repo root. `flutter/android/app/build.gradle` hardcodes `flutter { source '../..' }` (Flutter project root two levels up from the `app` module) and `main.proto.srcDirs += '../../../libs/hbb_common/protos'` (three levels up into `libs/hbb_common`). |
| `flutter/android` | Must stay exactly two levels below the Flutter project root (`flutter/`) for the Flutter Gradle plugin's `source '../..'` to resolve; also `../../../libs/hbb_common/protos` assumes `libs/` is three levels above `flutter/android/app/`. |
| `scripts/verify_enzu_config.py` | Hardcodes `SOURCE_OF_TRUTH = os.path.join("src", "enzu_config.rs")` resolved against the script's own parent-of-parent directory (i.e. assumes it lives at `<repo-root>/scripts/`). |
| `src/enzu_config.rs`, `build.rs` | Explicitly protected by the task's Phase 3 instruction and independently confirmed by the audit: `build.rs` injects `cargo:rustc-env` values consumed via `option_env!` in `src/enzu_config.rs`; `scripts/verify_enzu_config.py` also reads this exact path as its single source of truth. Moving either requires synchronized changes to `build.rs`, `enzu_config.rs`'s own doc comments, and the verifier — no practical benefit for the risk. |

**Net effect:** the entire Rust workspace (`src/`, `libs/`), the Flutter project
(`flutter/`), and the native dependency manifest (`vcpkg.json`) form one tightly
cross-referenced physical unit anchored at the repo root. None of it moves this phase.

## 6. Migration stages (this phase and beyond)

1. **Stage 0 (this phase, done):** safety tag `enzu-pre-structure-migration`; audit
   completed; conceptual architecture documented (this file); `android/`, `windows/`,
   `core/`, `config/` created as **pointer-only** directories (README + links to the
   real implementation, no code moved or duplicated); unwanted automatic CI trigger
   disabled (`flutter-nightly.yml` cron — see [CI.md](CI.md)).
2. **Stage 1 (future, opt-in):** if/when a genuinely ENZU-owned Android or Windows
   script needs a home that isn't shared with Flutter's own platform runner
   (e.g. a signing helper, a packaging step), add it under `android/scripts/` or
   `windows/scripts/` — never move the existing `flutter/android`, `flutter/windows`.
3. **Stage 2 (future, requires a dedicated audit):** a physical `core/` extraction
   (moving `src/`+`libs/*` Rust code under `core/`) is possible only with a full,
   deterministic update of: `Cargo.toml` (`build =`, `[lib]`, `[[bin]]`, `[workspace]
   members`, `[patch.crates-io]`), `build.rs`'s own relative file references,
   `scripts/verify_enzu_config.py`'s hardcoded path, `flutter/windows/CMakeLists.txt`'s
   `../../target/...` reference, and `flutter/android/app/build.gradle`'s
   `../../../libs/hbb_common/protos` reference. Not attempted until a concrete need
   (e.g. adding a second platform target) justifies the risk.
4. **Stage 3 (future):** re-evaluate once upstream sync (see [UPSTREAM_SYNC.md](UPSTREAM_SYNC.md))
   has been run at least once against the new layout, so any physical move is tested
   against both ENZU's own build and a fresh upstream merge.

## 7. Upstream synchronization implications

- `libs/hbb_common` is a **submodule**, updated independently of the superproject via
  `git submodule update --remote` (or a submodule-pointer bump commit) — a normal
  `git merge upstream/master` on the superproject does **not** by itself update it.
  Any structural move of `libs/hbb_common` would also need to stay compatible with
  however upstream itself manages that submodule.
- Every file marked `U*` in [DIRECTORY_TREE.md](DIRECTORY_TREE.md) (`src/core_main.rs`,
  `src/flutter_ffi.rs`, `src/lib.rs`) carries a **minimal, single-line ENZU hook** into
  otherwise-upstream files. These are the files most likely to conflict on a future
  `git merge upstream/master`; see [UPSTREAM_SYNC.md](UPSTREAM_SYNC.md) for the full list
  and the recommended merge procedure.
- Because no ENZU work this phase touched `src/`, `libs/`, or `flutter/` physically,
  the merge-conflict surface against upstream is unchanged from the pre-migration
  baseline (`84636ba6a`).
