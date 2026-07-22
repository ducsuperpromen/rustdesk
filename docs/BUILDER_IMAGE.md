# ENZU Android Builder Image

A reusable, pinned Docker image that contains the full Android toolchain, so the
GitLab pipeline does almost no setup work: **pull image → checkout → generate bridge
→ build → upload**.

> Defined in [`Dockerfile.enzu-builder`](../Dockerfile.enzu-builder).
> **This image is intentionally NOT published.** You build it yourself and make it
> available to your runner. Nothing here publishes to a public registry.

---

## Why an image?

The old GitLab pipeline installed Rust, Flutter, NDK, the Android SDK, vcpkg and every
apt dependency **on every single run**. That is slow, non-reproducible (upstream
`rustup`/`flutter`/`apt` can move), and it duplicates the toolchain definition inside
CI YAML. Baking the toolchain into one pinned image makes every build:

- **Reproducible** — exact tool versions are frozen in an image digest.
- **Fast** — no multi-minute toolchain install per run (notably `cargo install
  flutter_rust_bridge_codegen`, which compiles from source).
- **Simple** — the pipeline YAML shrinks to the actual build steps.

## What's inside (all pinned)

| Component | Version |
|-----------|---------|
| Base OS | Ubuntu 24.04 |
| Java (JDK) | 17 |
| Rust | 1.75.0 (+ `aarch64-linux-android` target) |
| cargo-ndk | 3.1.2 |
| cargo-expand | 1.0.95 |
| flutter_rust_bridge_codegen | 1.80.1 |
| Flutter (app build) | 3.24.5 → `/opt/flutter` |
| Flutter (bridge codegen) | 3.22.3 → `/opt/flutter-bridge` |
| Android SDK | cmdline-tools + `platforms;android-33` + `build-tools;34.0.0` |
| Android NDK | r28c → `/opt/android-ndk` |
| CMake / Ninja / NASM / LLVM / Clang | apt (Ubuntu 24.04) |
| Python 3 | apt |
| vcpkg | bootstrapped at commit `120deac…` → `/opt/vcpkg` |
| Git | apt |

Key environment variables are pre-set: `ANDROID_SDK_ROOT`, `ANDROID_NDK_HOME`,
`VCPKG_ROOT`, `FLUTTER_HOME`, `BRIDGE_FLUTTER_HOME`, `JAVA_HOME`, and `PATH`.

## Build it

```bash
# from the repo root
docker build -f Dockerfile.enzu-builder -t enzu/rustdesk-builder:latest .
```

## Make it available to the GitLab runner

Pick whichever fits your setup — the pipeline references `enzu/rustdesk-builder:latest`
with `pull_policy: [if-not-present, always]`:

- **Local (shell/docker runner on your own host):** build the image on that host; the
  runner uses the local copy (`if-not-present`).
- **Private registry / GitLab Container Registry:**
  ```bash
  docker tag  enzu/rustdesk-builder:latest registry.gitlab.com/pos9014819/rust-desk/enzu-rustdesk-builder:latest
  docker push registry.gitlab.com/pos9014819/rust-desk/enzu-rustdesk-builder:latest
  ```
  then update the `image:` name in [`.gitlab-ci.yml`](../.gitlab-ci.yml).

## What the pipeline still does at run time (and why)

Two things are **not** baked into the image, on purpose, because they depend on the
checked-out source:

1. **Bridge generation** — `generated_bridge.dart` / `bridge_generated.rs` are derived
   from `src/flutter_ffi.rs`, which changes with the code. Tools are preinstalled;
   only the fast codegen runs per build.
2. **vcpkg `install`** — the exact dependency set comes from the repo's `vcpkg.json`
   baseline. vcpkg is bootstrapped in the image; `flutter/build_android_deps.sh`
   resolves deps at build time (cacheable via the GitLab job cache).

## Updating the image

When a pinned version changes in `.github/workflows/enzu-build.yml`, change the
matching `ENV` in `Dockerfile.enzu-builder`, rebuild, and re-tag. Keep the two in
lock-step — the GitHub workflow is the source of truth for versions; the image mirrors
them so hosted and containerised builds stay identical.

## Non-goals

- No app/branding/network/Rust/Flutter source changes.
- Not published to any public registry.
- Windows is **not** in this image — Windows builds on GitHub (see
  [ENZU_BUILD_SYSTEM.md](ENZU_BUILD_SYSTEM.md)).
