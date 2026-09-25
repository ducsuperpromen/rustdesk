# core/ — ENZU Remote Core (pointer)

This directory is a **conceptual placeholder** for the ENZU Remote Core (rendezvous,
relay, remote desktop, codec, input, encryption). It does not yet physically host
that code — see [docs/architecture/SOURCE_LAYOUT.md](../docs/architecture/SOURCE_LAYOUT.md)
for the full audit of why, and for the migration stages under which a physical move
could eventually happen.

## Where the real implementation lives today

| Subsystem | Where |
|---|---|
| Rendezvous (hbbs registration, hole-punch/relay negotiation) | [`src/rendezvous_mediator.rs`](../src/rendezvous_mediator.rs), [`libs/hbb_common/src/config.rs`](../libs/hbb_common/) |
| Relay | [`src/client.rs`](../src/client.rs), `libs/hbb_common` |
| Remote desktop (client + controlled/server side) | [`src/server/`](../src/server/), [`src/client.rs`](../src/client.rs), [`src/ui_session_interface.rs`](../src/ui_session_interface.rs) |
| Codec / video capture | [`libs/scrap/`](../libs/scrap/) |
| Input (keyboard, mouse, clipboard) | [`libs/enigo/`](../libs/enigo/), [`src/keyboard.rs`](../src/keyboard.rs), [`src/clipboard.rs`](../src/clipboard.rs) |
| Encryption / key handling | [`src/common.rs`](../src/common.rs) (`get_key`, `get_rs_pk`), RustDesk's own crypto in `libs/hbb_common` |
| ENZU embedded server config (single source of truth) | [`src/enzu_config.rs`](../src/enzu_config.rs), injected at build time by [`build.rs`](../build.rs) |

For the full data-flow picture (which file calls what, at which point in the
process), see [docs/architecture/ARCHITECTURE.md](../docs/architecture/ARCHITECTURE.md)
and [docs/architecture/NETWORK_FLOW.md](../docs/architecture/NETWORK_FLOW.md).

## Why it isn't physically here

`src/` is the crate root Cargo already resolves (`Cargo.toml`'s `[lib]`/`[[bin]]`
paths, `build.rs`'s own `src/platform/*.cc`/`.mm` references), and `libs/*` are
Cargo workspace members addressed by relative path from the repo root. `libs/hbb_common`
is additionally a **git submodule**, not a plain directory. A physical `core/`
extraction is possible in principle but requires a deterministic, all-at-once update
of `Cargo.toml`, `build.rs`, `scripts/verify_enzu_config.py`, and the Windows/Android
build files that reference `target/` and `libs/hbb_common` by relative path — see
[docs/architecture/SOURCE_LAYOUT.md](../docs/architecture/SOURCE_LAYOUT.md) §5–6 for
the exact list and the staged plan for attempting it later.
