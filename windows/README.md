# windows/ — ENZU Windows ownership (pointer)

This directory is a **conceptual placeholder**, not the physical Windows project.
It exists so ENZU's target architecture (see
[docs/architecture/SOURCE_LAYOUT.md](../docs/architecture/SOURCE_LAYOUT.md)) has a
`windows/` owner to point at, without moving the real, working Windows project.

## Where the real implementation lives

| What | Where |
|---|---|
| Windows Flutter runner (CMake, `Runner`) | [`flutter/windows/`](../flutter/windows/) |
| Desktop build orchestration (Windows + others) | [`build.py`](../build.py) |
| MSVC static-CRT `rustflags` | [`.cargo/config.toml`](../.cargo/config.toml) |
| Windows-only Rust dependency (`virtual_display`) | [`libs/virtual_display/`](../libs/virtual_display/) |
| CI: Windows x64 build job | [`.github/workflows/enzu-build.yml`](../.github/workflows/enzu-build.yml) (`windows-x64` job) |

## Why it isn't physically here

`flutter/windows/CMakeLists.txt` hardcodes the built Rust library's location as
`../../target/<profile>/librustdesk.dll` — two directory levels above
`flutter/windows/`, i.e. the Cargo `target/` output at the **repo root**. Moving
`flutter/windows` (or the Cargo workspace root) would break that reference and every
other path derived from it. See
[docs/architecture/SOURCE_LAYOUT.md](../docs/architecture/SOURCE_LAYOUT.md) §5 for the
full dependency audit. Preserving the working build takes priority over the directory
tree looking like the target architecture.

If a genuinely ENZU-owned Windows script or packaging step ever needs a home that
isn't shared with Flutter's own platform runner, it belongs under `windows/scripts/`
here — not inside `flutter/windows`.
