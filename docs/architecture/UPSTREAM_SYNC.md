# Upstream Sync Design

Phase 7 deliverable of the ENZU source-structure migration. This document designs —
but does **not** execute — the procedure for pulling future `rustdesk/rustdesk`
upstream commits into this fork. As of this migration, upstream is **not** merged;
`feature/enzu-custom-client` remains ~231 commits behind `rustdesk/rustdesk@master`
(baseline `84636ba6a`, safety tag `enzu-pre-structure-migration`).

## Why keep structural migration and upstream sync separate

The ENZU config mechanism (`build.rs` -> `cargo:rustc-env` -> `option_env!` ->
`src/enzu_config.rs`) and the ENZU CI layer (`enzu-build.yml`, `.gitlab-ci.yml`,
`docker/builder/*`) were built to be **additive**: new files, or single-line hooks in
a handful of upstream files. That design exists specifically so a future
`git merge upstream/master` has the smallest possible conflict surface. Running a
structural migration and an upstream merge in the same change would make it much
harder to tell whether a build break came from the reorganization or from upstream's
own changes — so they must happen as two separate, independently-verifiable steps.

## Recommended procedure

```
rustdesk upstream (github.com/rustdesk/rustdesk)
       |
       v
git fetch upstream
       |
       v
create branch: upstream-sync/<version-or-date>   (e.g. upstream-sync/2026-09-25)
       |
       v
merge or rebase feature/enzu-custom-client onto upstream/master in that branch
       |
       v
review conflicts (see "ENZU-owned files" below for where they'll concentrate)
       |
       v
ENZU config integrity check:  python3 scripts/verify_enzu_config.py
       |
       v
Android build   (android-arm64 job in enzu-build.yml, or local equivalent)
       |
       v
Windows build   (windows-x64 job in enzu-build.yml, or local equivalent)
       |
       v
only then: merge upstream-sync/<version-or-date> into feature/enzu-custom-client
```

Step-by-step:

1. `git fetch upstream` — requires an `upstream` remote pointed at
   `https://github.com/rustdesk/rustdesk` (add with `git remote add upstream ...` if
   not already configured).
2. `git checkout -b upstream-sync/<date> feature/enzu-custom-client`. Never sync
   directly on `feature/enzu-custom-client` or `master`.
3. **Merge vs. rebase:** prefer `git merge upstream/master` (not rebase). This is a
   long-lived fork with its own CI/config layer; rebasing would rewrite every ENZU
   commit's history against a moving upstream base and make future syncs harder to
   reason about. A merge commit cleanly marks "this is where upstream N was absorbed."
4. Resolve conflicts (see below for where to expect them).
5. Run `python3 scripts/verify_enzu_config.py` — must print `[ENZU-VERIFY] OK`. This
   catches both a broken `option_env!` wiring and any accidental duplication of the
   ENZU server/key values outside `src/enzu_config.rs`.
6. Build Android (`android-arm64-v8a`) and Windows (`x64`) — either by dispatching
   `enzu-build.yml` from the sync branch, or locally with the same commands (see
   [BUILD_FLOW.md](BUILD_FLOW.md)). Both must succeed before proceeding.
7. Only after config verification and both builds pass: merge
   `upstream-sync/<date>` into `feature/enzu-custom-client` (or open a PR for review
   first — recommended once this fork has more than one contributor).
8. Tag the result if useful for later bisection (e.g. `enzu-post-sync-<date>`), same
   convention as `enzu-pre-structure-migration`.

## ENZU-owned files requiring special attention during upstream merges

These are exactly the files [DIRECTORY_TREE.md](DIRECTORY_TREE.md) marks `E` (new) or
`U*` (upstream file with a minimal ENZU hook) — the fork's entire merge-conflict
surface with upstream lives here:

| File | Why it needs attention |
|---|---|
| `src/lib.rs` | `U*` — one added line, `pub mod enzu_config;`. If upstream reorders/rewrites the module list, re-add this line rather than accepting either side blindly. |
| `src/core_main.rs` | `U*` — one added call to `enzu_config::apply_enzu_defaults()` near the desktop entry point (`core_main()`, ~line 35 at baseline). Upstream changes to `core_main()`'s control flow are the most likely source of a real conflict here; re-verify the call still runs before the first rendezvous/config lookup. |
| `src/flutter_ffi.rs` | `U*` — same hook, called from `initialize()` for the Flutter/mobile entry path. |
| `src/enzu_config.rs` | `E` — pure ENZU file, upstream will never touch it, but double-check its two upstream dependencies still exist after a merge: `hbb_common::config::{keys, DEFAULT_SETTINGS}` and `crate::common::get_rs_pk`. A rename of either in `libs/hbb_common` or `src/common.rs` will fail `enzu_config.rs`'s own unit tests. |
| `build.rs` | `E`-appended — the ENZU config-injection block (`set_enzu_build_config` and helpers) was added at the end of an otherwise-upstream file; a conflict here usually means upstream edited `fn main()` or the Android/Windows/macOS helper functions above the ENZU block — keep the ENZU block intact and re-attach its call in `main()`. |
| `.gitignore` | `U*` — ENZU added private-key guards (`id_ed25519`, `**/id_ed25519`) and `.env`. Re-add if upstream's own `.gitignore` changes conflict. |
| `scripts/verify_enzu_config.py` | `E` — pure ENZU, but it parses `src/enzu_config.rs` with a regex (`extract_const`); if the constant declarations' syntax changes, update the regex. |
| `.env.example`, `docs/enzu/EMBEDDED_SERVER_CONFIG.md` | `E` — documentation/template, no code dependency, but keep in sync if the config keys ever change. |
| `.github/workflows/enzu-build.yml`, `.gitlab-ci.yml`, `docker/builder/*` | `E` — additive CI, named so it can never collide with an upstream workflow filename. Upstream toolchain version bumps (Flutter/Rust/NDK/vcpkg versions in `flutter-build.yml`/`bridge.yml`) should be manually mirrored into `enzu-build.yml`'s pinned `env:` block so ENZU builds stay reproducible against the same toolchain upstream uses — this file intentionally does **not** inherit from upstream's workflows. |
| `libs/hbb_common` | **Git submodule**, not a regular tracked directory (see [SOURCE_LAYOUT.md](SOURCE_LAYOUT.md) §8 and `.gitmodules`). A `git merge upstream/master` updates the *submodule pointer* (the commit it's pinned to) like any other file, but pulling the new submodule commit itself requires a separate `git submodule update --remote` (or checking out that commit inside `libs/hbb_common`) — don't assume a plain merge brings in `hbb_common`'s own upstream changes. |

## What a sync does *not* need to touch

Everything under `docs/architecture/`, `docs/enzu/`, `docs/ENZU_BUILD_SYSTEM.md`, and
`docs/BUILDER_IMAGE.md` is pure ENZU documentation with no upstream counterpart —
never a merge conflict, only a place to record what changed if the sync affects the
architecture described in them.

## Current state (as of this migration)

- No fetch, merge, or rebase against upstream was performed as part of this
  migration — per the task's explicit instruction, upstream sync is out of scope for
  this phase.
- This fork is still ~231 commits behind `rustdesk/rustdesk@master` (as stated in the
  migration brief); the first real sync should be planned as its own reviewed change,
  following the procedure above.
