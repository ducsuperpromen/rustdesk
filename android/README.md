# android/ — ENZU Android ownership (pointer)

This directory is a **conceptual placeholder**, not the physical Android project.
It exists so ENZU's target architecture (see
[docs/architecture/SOURCE_LAYOUT.md](../docs/architecture/SOURCE_LAYOUT.md)) has an
`android/` owner to point at, without moving the real, working Android project.

## Where the real implementation lives

| What | Where |
|---|---|
| Android Gradle project (Kotlin, manifest, resources) | [`flutter/android/`](../flutter/android/) |
| Native lib JNI glue | `flutter/android/app/src/main/kotlin/**/ffi.kt` |
| Native lib placement (build output) | `flutter/android/app/src/main/jniLibs/arm64-v8a/` |
| Rust -> Android cross-compile (cargo-ndk) | [`flutter/ndk_arm64.sh`](../flutter/ndk_arm64.sh) (and the other `ndk_*.sh` variants) |
| Android native dependency build (vcpkg) | [`flutter/build_android_deps.sh`](../flutter/build_android_deps.sh) |
| CI: Android arm64-v8a build job | [`.github/workflows/enzu-build.yml`](../.github/workflows/enzu-build.yml) (`android-arm64` job), [`.gitlab-ci.yml`](../.gitlab-ci.yml) |

## Why it isn't physically here

`flutter/android/app/build.gradle` resolves the Flutter project root and the
`hbb_common` proto directory via **relative paths** (`flutter { source '../..' }`,
`main.proto.srcDirs += '../../../libs/hbb_common/protos'`). Moving `flutter/android`
out from under `flutter/` would break both without an invasive Gradle rewrite — see
[docs/architecture/SOURCE_LAYOUT.md](../docs/architecture/SOURCE_LAYOUT.md) §5 for the
full dependency audit. Preserving the working build takes priority over the
directory tree looking like the target architecture.

If a genuinely ENZU-owned Android script or asset ever needs a home that isn't shared
with Flutter's own platform runner (e.g. a signing helper that has nothing to do with
the Flutter build), it belongs under `android/scripts/` here — not inside
`flutter/android`.
