# ENZU Build System

This document describes the ENZU custom-client CI/CD layer for this RustDesk fork.

> **Scope**: infrastructure only. The application is built **exactly** as upstream
> builds it (identical toolchain versions and build scripts). No branding, server,
> relay, key, package, app-name, icon, or feature changes are made. The only
> difference from upstream CI is *which* platforms are built and that ENZU CI
> **uploads artifacts only — it never publishes releases**.

---

## 1. Architecture

Two isolated, ENZU-specific pipelines are added. They are purely additive and never
modify upstream workflow files, so `git merge upstream/master` stays conflict-free
(upstream never touches `enzu-*` / `.gitlab-ci.yml`).

```
                       feature/enzu-custom-client
                                  │
         ┌────────────────────────┴────────────────────────┐
         │                                                  │
  GitHub Actions                                        GitLab CI
  .github/workflows/enzu-build.yml                     .gitlab-ci.yml
  (workflow_dispatch)                                  (feature branch / manual)
         │                                                  │
   ┌─────┴──────┐                                    ┌──────┴───────┐
   │ generate-  │  (shared, platform-neutral)        │ android-     │  (inline bridge)
   │ bridge     │                                    │ arm64        │
   └─────┬──────┘                                    └──────────────┘
         │                                                  │  (independent)
   ┌─────┴───────────────┐                           ┌──────┴───────┐
   │                     │                           │ windows-x64  │  (inline bridge,
┌──┴─────────┐   ┌───────┴──────┐                    │  manual)     │   tagged runner)
│ android-   │   │ windows-x64  │                    └──────────────┘
│ arm64      │   │              │
└────────────┘   └──────────────┘
   (parallel — neither depends on the other)
```

**Dependency rules enforced:**

- Android does **not** depend on Windows (and vice-versa).
- Android does **not** depend on any Linux/macOS *desktop platform build* — those
  platforms are not built at all in ENZU CI.
- On GitHub, both platform jobs share one neutral `generate-bridge` prerequisite
  (the flutter-rust-bridge codegen). This is a codegen helper, **not** a platform
  build; it is exactly how upstream structures bridge generation.
- On GitLab, each job generates its bridge **inline**, so the two jobs are fully
  independent with zero shared stages.
- Every artifact uploads independently, per job.

---

## 2. GitHub workflow (`.github/workflows/enzu-build.yml`)

- **Trigger:** `workflow_dispatch` (manual only).
- **Permissions:** `contents: read` (cannot create releases/tags).
- **Jobs:**
  1. `generate-bridge` — `ubuntu-22.04`, Flutter 3.22.3, produces `enzu-bridge-artifact`.
  2. `android-arm64` — `ubuntu-24.04`, `needs: [generate-bridge]`.
  3. `windows-x64` — `windows-2022`, `needs: [generate-bridge]`.
- **Output:** artifacts only. No `softprops/action-gh-release`, no tags, no releases.

---

## 3. GitLab workflow (`.gitlab-ci.yml`)

- **Stages:** `android` (first), then `windows` (second).
- `android-arm64` runs on a **Linux** runner (tag `linux`), image `ubuntu:24.04`.
  Self-contained: installs Rust/Flutter/NDK/vcpkg, generates the bridge inline,
  builds the lib and APK.
- `windows-x64` runs on a **tagged Windows** runner (tags `windows`, `x64`),
  `needs: []`, `when: manual` — so the pipeline stays green when no Windows runner
  is registered.
- Runs from `feature/enzu-custom-client`. Does **not** merge or pull gitlab `main`
  and does **not** touch the initial README commit.

---

## 4. Required SDK / tool versions (pinned to upstream)

| Tool | Version | Used by |
|------|---------|---------|
| Rust | `1.75` | Android lib, Windows, bridge |
| cargo-ndk | `3.1.2` | Android lib |
| Android NDK | `r28c` | Android lib + `libc++_shared.so` |
| Flutter (app build) | `3.24.5` | Android + Windows app |
| Flutter (bridge codegen) | `3.22.3` | bridge only |
| flutter_rust_bridge_codegen | `1.80.1` | bridge |
| cargo-expand | `1.0.95` | bridge |
| LLVM/Clang | `15.0.6` | Windows |
| vcpkg commit | `120deac3062162151622ca4860575a33844ba10b` | native deps |
| Java (JDK) | `17` | Android Gradle |
| App VERSION | `1.4.9` | artifact naming |

---

## 5. Required runners

| Job | GitHub runner | GitLab runner |
|-----|---------------|---------------|
| generate-bridge | `ubuntu-22.04` | (inline per job) |
| Android arm64 | `ubuntu-24.04` | Linux, tag `linux`, Docker `ubuntu:24.04` |
| Windows x64 | `windows-2022` | self-hosted shell runner, tags `windows`+`x64`, with Git, Python 3, VS Build Tools |

---

## 6. Artifact locations

| Platform | Artifact name | Contents | Retention |
|----------|---------------|----------|-----------|
| Android | `enzu-rustdesk-android-arm64` | `enzu-rustdesk-1.4.9-android-arm64.apk` (arm64-v8a, debug-signed) | 14 days |
| Windows | `enzu-rustdesk-windows-x64` | `rustdesk/` portable folder (`rustdesk.exe` + deps) | 14 days |
| Bridge (GitHub) | `enzu-bridge-artifact` | generated bridge Dart/Rust files | 14 days |

Download from the run summary (GitHub Actions → run → Artifacts; GitLab → pipeline
job → Browse/Download artifacts).

---

## 7. Android build path (arm64-v8a)

1. Free disk space, install apt build deps, Java 17.
2. Checkout with recursive submodules.
3. Install Flutter 3.24.5, apply dropdown-filter patch.
4. Install NDK r28c.
5. Setup vcpkg; run `flutter/build_android_deps.sh arm64-v8a`.
6. Restore/generate bridge files.
7. Install Rust 1.75, `rustup target add aarch64-linux-android`, install cargo-ndk 3.1.2.
8. `flutter/ndk_arm64.sh` builds `liblibrustdesk.so`.
9. Copy `librustdesk.so` and NDK `libc++_shared.so` into `jniLibs/arm64-v8a`.
10. Debug signing (`signingConfigs.release` → `signingConfigs.debug`), Gradle mem bump.
11. `flutter build apk --release --target-platform android-arm64 --split-per-abi`.
12. Rename to `enzu-rustdesk-1.4.9-android-arm64.apk` and upload.

## 8. Windows build path (x64)

1. Checkout with recursive submodules; restore/generate bridge.
2. Install LLVM 15.0.6, Flutter 3.24.5 (x64), replace with RustDesk custom engine.
3. Apply dropdown-filter patch.
4. Install Rust 1.75 (`x86_64-pc-windows-msvc`).
5. Setup vcpkg; install `x64-windows-static` deps.
6. `python3 build.py --portable --flutter --skip-portable-pack --hwcodec --vram`.
7. Move `flutter/build/windows/x64/runner/Release` → `rustdesk/`; add usbmmidd +
   (best-effort) printer driver.
8. Upload `rustdesk/` folder.

---

## 9. Cache policy

Caching is **best-effort and never fatal**:

- GitHub `Swatinem/rust-cache` and `actions/cache` steps use `continue-on-error: true`.
- vcpkg GitHub Actions binary cache (`VCPKG_BINARY_SOURCES: clear;x-gha,readwrite`)
  is a read/write *binary* cache — on a cache-service error vcpkg logs a warning and
  **rebuilds from source** rather than failing.
- GitLab `cache:` blocks use `when: always` with `policy: pull-push`; a miss just
  triggers a clean build.

Real compiler/toolchain errors (Cargo, Gradle, Flutter, CMake, vcpkg source build)
are **never** suppressed.

---

## 10. Troubleshooting

**Cache failures** (`HTTP 400`, "cache service unavailable", "too many retries")
— treat as infrastructure noise, not source errors. Builds proceed from a clean
state. If vcpkg `x-gha` 400s persist and slow things down, set
`VCPKG_BINARY_SOURCES: "clear"` in the workflow `env` to disable the binary cache
entirely (slower, fully clean).

**Bridge failures** (`flutter_rust_bridge_codegen` errors, missing
`generated_bridge.freezed.dart`) — the bridge must be generated with Flutter
**3.22.3** and `extended_text` downgraded to `13.0.0`; check those steps ran and
that `flutter pub get` succeeded before codegen. On GitHub, re-run only the
`generate-bridge` job.

**Cargo failures** — confirm Rust `1.75` (not a newer default), that
`rustup target add` ran for the target, and that submodules are present
(`hbb_common` etc.). Clear the (optional) Rust cache and retry.

**Gradle failures** — ensure `JAVA_HOME` points to JDK 17 and the Gradle memory
bump applied. Native `librustdesk.so` and `libc++_shared.so` must exist in
`jniLibs/arm64-v8a` before `flutter build apk`.

**Flutter failures** — verify Flutter `3.24.5` for the app build and that the
dropdown-filter patch applied. On Windows, confirm the custom engine replacement
step succeeded.

**Submodule failures** — always checkout with recursive submodules
(`submodules: recursive` on GitHub, `GIT_SUBMODULE_STRATEGY: recursive` on GitLab).
A missing submodule surfaces later as a Cargo/CMake "file not found".

**Windows runner absent (GitLab)** — the `windows-x64` job is `when: manual` with
`needs: []`; the pipeline succeeds without it. Register a shell runner tagged
`windows`+`x64` (Git, Python 3, VS Build Tools) and start the job manually.

---

## 11. Remotes

```
github   -> git@github.com:ducsuperpromen/rustdesk.git   (origin renamed to github)
gitlab   -> git@gitlab.com:pos9014819/rust-desk.git
upstream -> https://github.com/rustdesk/rustdesk.git      (kept, never removed)
```

Push the ENZU branch to both `github` and `gitlab` as `feature/enzu-custom-client`.
Never push to / merge gitlab `main`.
