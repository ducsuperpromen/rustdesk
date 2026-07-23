# ENZU Android Builder Image

A reusable, pinned Docker image containing the full Android toolchain, so the GitLab
pipeline does almost no setup: **pull image → checkout → verify config → generate
bridge → build → upload**.

- Image name/tag: **`enzu/rustdesk-builder:1.0.0`** (pinned; never `:latest`).
- Version is stored in [`VERSION`](VERSION) and is the single source for the tag.
- Dockerfile: [`Dockerfile`](Dockerfile).

> **Intentionally NOT published.** You build it yourself and make it available to your
> runner. Nothing here publishes to a public registry.

---

## Why an image?

The old GitLab pipeline installed Rust, Flutter, NDK, the Android SDK, vcpkg and every
apt dependency **on every run** — slow, non-reproducible (upstream `rustup`/`flutter`/
`apt` move), and it duplicated the toolchain in CI YAML. Baking a pinned toolchain into
one image makes every build **reproducible** (frozen versions), **fast** (no multi-minute
`cargo install flutter_rust_bridge_codegen` per run), and keeps the pipeline YAML small.

## What's inside (all pinned)

| Component | Version |
|-----------|---------|
| Base OS | Ubuntu 24.04 |
| Java (JDK) | 17 |
| Rust | 1.75.0 (+ `aarch64-linux-android`) |
| cargo-ndk | 3.1.2 |
| cargo-expand | 1.0.95 |
| flutter_rust_bridge_codegen | 1.80.1 |
| Flutter (app build) | 3.24.5 → `/opt/flutter` |
| Flutter (bridge codegen) | 3.22.3 → `/opt/flutter-bridge` |
| Android SDK | cmdline-tools + `platforms;android-33` + `build-tools;34.0.0` |
| Android NDK | r28c → `/opt/android-ndk` |
| CMake / Ninja / NASM / LLVM / Clang | apt (Ubuntu 24.04) |
| Python 3 | apt (runs `scripts/verify_enzu_config.py`) |
| vcpkg | bootstrapped at commit `120deac…` → `/opt/vcpkg` |
| Git | apt |

Env pre-set: `ANDROID_SDK_ROOT`, `ANDROID_NDK_HOME`, `VCPKG_ROOT`, `FLUTTER_HOME`,
`BRIDGE_FLUTTER_HOME`, `JAVA_HOME`, `PATH`.

## Build it

From anywhere (scripts resolve the repo root and read the tag from `VERSION`):

```bash
docker/builder/build.sh          # Linux/macOS
# or
docker/builder/build.ps1         # Windows PowerShell
# equivalently:
docker build -f docker/builder/Dockerfile -t enzu/rustdesk-builder:1.0.0 .
```

## Make it available to the GitLab runner

`.gitlab-ci.yml` references `enzu/rustdesk-builder:1.0.0` with
`pull_policy: [if-not-present, always]`:

- **Local (shell/docker runner on your host):** build on that host; the runner uses the
  local copy.
- **Private registry / GitLab Container Registry:** publish with the optional helper
  (never run by CI, and it never logs tokens — you `docker login` yourself first):
  ```bash
  docker/builder/push.sh registry.gitlab.com/pos9014819/rust-desk
  ```
  then update the `image:` name in `.gitlab-ci.yml` and keep the `:1.0.0` tag.

## Updating the image (bump the version)

1. Edit `VERSION` (e.g. `1.0.1`) and change the pinned versions in `Dockerfile`.
2. Rebuild with `build.sh` / `build.ps1`.
3. Update the `:x.y.z` tag referenced in `.gitlab-ci.yml` and docs.

The GitHub workflow (`.github/workflows/enzu-build.yml`) remains the source of truth for
tool versions; keep the Dockerfile's `ENV` block in lock-step so hosted and containerised
builds stay identical.

## What runs at build time (not baked in)

- **ENZU config verification** — `scripts/verify_enzu_config.py` (fast, deterministic).
- **Bridge generation** — derived from `src/flutter_ffi.rs`; tools are preinstalled so
  only the fast codegen runs.
- **vcpkg `install`** — dependency set comes from the repo's `vcpkg.json`; vcpkg is
  bootstrapped in the image, deps resolve at build time.

## Non-goals

- No app/branding/network/Rust/Flutter source changes.
- Not published to any public registry.
- Windows is **not** in this image — Windows builds on GitHub (see
  [../../docs/ENZU_BUILD_SYSTEM.md](../../docs/ENZU_BUILD_SYSTEM.md)).
