# ENZU Build System

This document describes the ENZU custom-client CI/CD layer for this RustDesk fork.

> **Scope**: infrastructure only. The application is built **exactly** as upstream
> builds it (identical toolchain versions and build scripts). No branding, server,
> relay, key, package, app-name, icon, or feature changes are made. ENZU CI
> **uploads artifacts only — it never publishes releases**.

---

## 1. CI philosophy

- **Additive, never invasive.** Every ENZU file (`enzu-*`, `.gitlab-ci.yml`,
  `docker/builder/*`, `docs/*`) is new. No upstream workflow or app source is
  modified, so `git merge upstream/master` stays conflict-free.
- **One platform, one owner.** Each platform is owned by the CI that runs it best,
  avoiding duplicated infrastructure.
- **Reproducible over convenient.** Tool versions are pinned everywhere and, for
  Android on GitLab, frozen into a Docker image.
- **Caches are best-effort.** A cache miss or cache-service outage must never fail a
  build; real compiler errors are never suppressed.
- **Artifacts only.** No releases, no tags — `permissions: contents: read` on GitHub.

### Why GitHub builds Windows
Windows builds run cleanly on GitHub's hosted `windows-2022` runners with zero extra
infrastructure. Reproducing that on GitLab would require a self-hosted Windows runner
plus Visual Studio, LLVM, Flutter, Rust and vcpkg — high cost, no benefit.

### Why GitLab builds Android
Android is a Linux-native build. GitLab runs it on a Linux runner using a prebuilt
**builder image** (see [docker/builder/README.md](../docker/builder/README.md)) that already contains the
whole toolchain, giving fast, reproducible builds. GitHub also builds Android on
hosted `ubuntu-24.04` as an always-available fallback.

---

## 2. Architecture

```
                       feature/enzu-custom-client
                                  │
         ┌────────────────────────┴─────────────────────────┐
   GitHub Actions                                        GitLab CI
   .github/workflows/enzu-build.yml                     .gitlab-ci.yml
   (workflow_dispatch, contents: read)                  (feature branch / manual)
         │                                                    │
   generate-bridge  (ubuntu-22.04, Flutter 3.22.3)      image: enzu/rustdesk-builder
         │                                                    │
   ┌─────┴───────────────┐                              ┌─────┴──────┐
   │                     │                              │ android-   │
┌──┴─────────┐   ┌───────┴──────┐                       │ arm64      │
│ android-   │   │ windows-x64  │                       └────────────┘
│ arm64      │   │              │                       (bridge generated inline
└────────────┘   └──────────────┘                        using tools baked in image)
   (parallel — neither depends on the other)
```

**Dependency rules enforced:**

- Android and Windows never depend on each other.
- No Linux/macOS *desktop platform build* exists in ENZU CI, so Android cannot depend
  on one.
- On GitHub, both platform jobs share one neutral `generate-bridge` prerequisite —
  a codegen helper, not a platform build (see §7).
- Every artifact uploads independently, per job.

---

## 3. GitHub workflow (`.github/workflows/enzu-build.yml`)

- **Trigger:** `workflow_dispatch` (manual only).
- **Permissions:** `contents: read` (cannot create releases/tags).
- **Jobs:**
  1. `generate-bridge` — `ubuntu-22.04`, Flutter 3.22.3, produces `enzu-bridge-artifact`.
  2. `android-arm64` — `ubuntu-24.04`, `needs: [generate-bridge]`.
  3. `windows-x64` — `windows-2022`, `needs: [generate-bridge]`.
- **Output:** artifacts only. No release actions, no tags.

## 4. GitLab workflow (`.gitlab-ci.yml`)

- **Single stage `build`, single job `android-arm64`.** Windows is not built here.
- Runs on the **builder image** (`enzu/rustdesk-builder:1.0.0`, pinned), Linux+docker runner.
- Runs `scripts/verify_enzu_config.py` before building (also a dedicated GitHub job).
- Self-contained: generates the bridge inline (tools from the image), builds the lib
  and APK, uploads the APK (14-day retention).
- Runs from `feature/enzu-custom-client`; never merges/pulls gitlab `main`.

### Build-time server configuration

The ENZU ID/relay servers and public key are **injected at build time** (precedence:
CI Variable → local `.env` → built-in ENZU fallback in `src/enzu_config.rs`). CI never
stores these values in YAML — GitHub maps repo *Variables*, GitLab auto-injects CI/CD
*Variables*; `build.rs` validates them and fails fast on a provided-but-invalid value.
Full details: [enzu/EMBEDDED_SERVER_CONFIG.md](enzu/EMBEDDED_SERVER_CONFIG.md).

---

## 5. Required SDK / tool versions (pinned to upstream)

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

The GitHub workflow is the **source of truth** for these versions; the builder image
`ENV` block mirrors them.

## 6. Required runners

| Job | GitHub runner | GitLab runner |
|-----|---------------|---------------|
| generate-bridge | `ubuntu-22.04` | (n/a — inline in image) |
| Android arm64 | `ubuntu-24.04` | Linux+docker, tags `linux`,`docker`, builder image |
| Windows x64 | `windows-2022` | (not built on GitLab) |

## 7. Bridge generation rationale

flutter-rust-bridge codegen is required (the generated files are git-ignored). It must
run with **Flutter 3.22.3** and `extended_text` downgraded to `13.0.0` — the exact
combination upstream pins for a reproducible `generated_bridge.freezed.dart`. Changing
it risks a different/failing freezed output, so it is kept verbatim.

**On GitHub — shared `generate-bridge` job (kept).** There are two consumers
(Android + Windows). Bridge codegen is expensive (`cargo install
flutter_rust_bridge_codegen` compiles from source) and version-sensitive. Running it
**once** and sharing the artifact is cheaper and gives a single source of truth. The
only cost is one neutral prerequisite edge, which does not couple the two platforms to
each other. Verdict: **shared is objectively better here; keep it.**

**On GitLab — inline (one consumer).** GitLab builds only Android, so there is a single
consumer and no artifact to share. Generating inline (using tools already baked into
the image, so it's fast) is simpler and removes cross-job coupling entirely.

> This is a deliberate, asymmetric choice: shared where there are 2 consumers, inline
> where there is 1. The bridge command itself is duplicated between the GitHub job and
> the GitLab script (minor tech debt — see §12).

## 8. Artifact locations

| Platform | Artifact name | Contents | Retention |
|----------|---------------|----------|-----------|
| Android | `enzu-rustdesk-android-arm64-configured` | `enzu-rustdesk-1.4.9-android-arm64.apk` (arm64-v8a, debug-signed, ENZU defaults) | 14 days |
| Windows | `enzu-rustdesk-windows-x64` | `rustdesk/` portable folder | 14 days |
| Bridge (GitHub) | `enzu-bridge-artifact` | generated bridge Dart/Rust files | 14 days |

## 9. Build paths

**Android arm64-v8a:** deps + Java 17 → Flutter 3.24.5 + patch → NDK r28c → vcpkg +
`build_android_deps.sh arm64-v8a` → bridge → Rust 1.75 + cargo-ndk 3.1.2 +
`ndk_arm64.sh` → copy `librustdesk.so` + `libc++_shared.so` into `jniLibs/arm64-v8a` →
debug signing → `flutter build apk --release --target-platform android-arm64
--split-per-abi`.

**Windows x64:** bridge → LLVM 15.0.6 + Flutter 3.24.5 (x64) + custom engine + patch →
Rust 1.75 (`x86_64-pc-windows-msvc`) → vcpkg `x64-windows-static` → `python3 build.py
--portable --flutter --skip-portable-pack --hwcodec --vram` → `rustdesk/` folder.

## 10. Cache policy

Caching is **best-effort and never fatal**:

- GitHub `Swatinem/rust-cache` and `actions/cache` (Gradle, bridge) steps use
  `continue-on-error: true`; Flutter installs use `cache: true`.
- vcpkg GitHub Actions binary cache (`VCPKG_BINARY_SOURCES: clear;x-gha,readwrite`) —
  on a cache-service error vcpkg logs a warning and **rebuilds from source**.
- GitLab `cache:` uses `when: always`; a miss just triggers a clean build. Cargo
  registry + `target/` are cached inside the project dir.

Real compiler/toolchain errors (Cargo, Gradle, Flutter, CMake, vcpkg) are never
suppressed.

## 11. Troubleshooting

**Cache failures** (`HTTP 400`, "cache service unavailable", "too many retries") —
infrastructure noise, not source errors; builds proceed from a clean state. If vcpkg
`x-gha` 400s persist, set `VCPKG_BINARY_SOURCES: "clear"` to disable the binary cache.

**Bridge failures** — must use Flutter 3.22.3 + `extended_text 13.0.0`; verify
`flutter pub get` ran before codegen. On GitHub, re-run only `generate-bridge`.

**Cargo failures** — confirm Rust `1.75`, that `rustup target add` ran, and submodules
are present. Clear the (optional) Rust cache and retry.

**Gradle failures** — ensure `JAVA_HOME` is JDK 17 and the memory bump applied;
`librustdesk.so` + `libc++_shared.so` must exist in `jniLibs/arm64-v8a` first.

**Flutter failures** — verify Flutter 3.24.5 for the app build and the dropdown patch;
on Windows confirm the custom-engine replacement succeeded.

**Submodule failures** — always checkout recursively (`submodules: recursive` /
`GIT_SUBMODULE_STRATEGY: recursive`).

**GitLab image missing** — build `enzu/rustdesk-builder:1.0.0` and make it available to
the runner ([docker/builder/README.md](../docker/builder/README.md)).

## 12. Future scaling & known tech debt

- **Bridge command duplication** — the codegen invocation lives in both the GitHub
  `generate-bridge` job and the GitLab script. If it grows, extract an ENZU-specific
  shell script referenced by both (kept inline today for simplicity/low merge risk).
- **New ABIs** — add a matrix entry (Android) or a target; the builder image already
  has the arm64 toolchain (extend `rustup target add` / vcpkg triplets for more).
- **Release signing** — currently debug-signed; wire `ANDROID_SIGNING_KEY` &
  friends when release artifacts are needed (opt-in, out of scope here).
- **vcpkg caching on GitLab** — deps resolve at run time; a project-dir binary cache
  could be added if build minutes matter.

## 13. Developer onboarding

1. **Clone & remotes** (standard layout — `origin` kept for tooling compatibility):
   ```
   origin   -> git@github.com:ducsuperpromen/rustdesk.git
   gitlab   -> git@gitlab.com:pos9014819/rust-desk.git
   upstream -> https://github.com/rustdesk/rustdesk.git
   ```
2. **Work on** `feature/enzu-custom-client`. Never push to / merge `main` on either
   remote.
3. **GitHub build:** Actions → “ENZU Build” → *Run workflow* (`workflow_dispatch`).
   Download APK/Windows artifacts from the run summary.
4. **GitLab build:** build the builder image once
   (`docker/builder/build.sh`, produces `enzu/rustdesk-builder:1.0.0`), make
   it available to a Linux+docker runner, then push the branch to `gitlab`.
5. **Bump a tool version:** edit `.github/workflows/enzu-build.yml` (source of truth),
   then mirror it in `docker/builder/Dockerfile`.

---

## Appendix: root-cause note on the earlier CI failures

The earlier failures (`HTTP 400`, "our services aren't available", "too many retries")
came from the **original large upstream workflow** run. The logs for that run are not
available in this environment (`gh` CLI is not installed / not authenticated here), so
the first-failing step **cannot be proven** from data. Observationally the messages are
GitHub cache/artifact *service* errors (infrastructure), not compiler errors — but this
remains unproven until the run logs are inspected. The ENZU workflows are hardened
against that class of failure regardless (see §10), which is the actionable mitigation
independent of the unproven root cause.
