# config/ — ENZU configuration (pointer)

This directory is a **conceptual placeholder** for ENZU-owned configuration. The
actual configuration mechanism is not moved here — it stays exactly where it is,
per the task's explicit instruction:

> DO NOT move `src/enzu_config.rs` if moving it would complicate Rust module
> resolution for no practical benefit. DO NOT move `build.rs`.

## Where the real implementation lives

| What | Where |
|---|---|
| Single source of truth for embedded server config (fallback constants + apply logic) | [`src/enzu_config.rs`](../src/enzu_config.rs) |
| Build-time injection (`cargo:rustc-env` from CI env / `.env`) | [`build.rs`](../build.rs) (`set_enzu_build_config` and helpers) |
| Local-dev config template (git-ignored `.env` is copied from this) | [`.env.example`](../.env.example) |
| Deterministic verifier (run in CI before any platform build) | [`scripts/verify_enzu_config.py`](../scripts/verify_enzu_config.py) |
| Full reference documentation | [`docs/enzu/EMBEDDED_SERVER_CONFIG.md`](../docs/enzu/EMBEDDED_SERVER_CONFIG.md) |

Precedence (unchanged by this migration): **CI env var > repo-root `.env` (local dev)
> built-in ENZU fallback constant**, resolved in `build.rs` and read at compile time
via `option_env!()` in `src/enzu_config.rs`.

## Why it isn't physically here

`build.rs` must live at the Cargo package root (Cargo's default `build =` location);
`src/enzu_config.rs` is read by both `build.rs`'s injected `cargo:rustc-env` values
and by `scripts/verify_enzu_config.py`'s hardcoded path
(`os.path.join("src", "enzu_config.rs")`). Moving either would require synchronized,
non-trivial changes across all three for no functional benefit — see
[docs/architecture/SOURCE_LAYOUT.md](../docs/architecture/SOURCE_LAYOUT.md) §5.
