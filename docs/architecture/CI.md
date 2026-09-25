# CI Audit — GitHub Actions Workflows

Phase 6 deliverable of the ENZU source-structure migration. Audits every workflow in
`.github/workflows/` for its trigger, purpose, and whether it costs this fork Actions
minutes automatically (i.e. without a human clicking "Run workflow").

## Workflow table

| Workflow | Trigger(s) | Purpose | ENZU status | Auto-run on this fork? |
|---|---|---|---|---|
| `enzu-build.yml` | `workflow_dispatch` | ENZU production build: verify config -> generate bridge -> Android arm64-v8a + Windows x64. Artifacts only, no releases. | **ENZU REQUIRED** | No — manual only. |
| `ci.yml` | `workflow_dispatch`, `pull_request`, `push` (branches: `master`, with `paths-ignore`) | Upstream general CI (lint/build checks). | UPSTREAM LEGACY | Yes, but only for PRs and pushes to **`master`** — this fork develops on `feature/enzu-custom-client`, so it fires only if/when that branch is merged to `master` or a PR targets `master`. |
| `bridge.yml` | `workflow_call` only | Reusable job: generates the flutter-rust-bridge files for multiple platforms. Not directly triggerable. | UPSTREAM LEGACY (reusable) | No — only runs when called by another workflow (e.g. `flutter-build.yml`). |
| `flutter-build.yml` | `workflow_call` only | Reusable: full upstream multi-platform Flutter build (Android/iOS/Windows/macOS/Linux/web). Not directly triggerable. | UPSTREAM LEGACY (reusable) | No — only runs when called (by `flutter-nightly.yml` or `flutter-tag.yml`). |
| `flutter-ci.yml` | `workflow_dispatch`, `pull_request`, `push` (branches: `master`, with `paths-ignore`) | Upstream Flutter-specific CI checks. | UPSTREAM LEGACY | Same as `ci.yml` — only on PR/push targeting `master`. |
| `flutter-nightly.yml` | ~~`schedule` (`0 0 * * *`)~~ **disabled this phase**, `workflow_dispatch` | Nightly build of every upstream platform via `flutter-build.yml`, uploaded under the `nightly` tag. | UPSTREAM LEGACY — **AUTO-RUN UNWANTED**, now fixed | **Was** yes (every day, any branch, via the default branch's copy of the workflow — GitHub schedules always run the version of the file on the repo's default branch). **Now** no — cron trigger commented out, `workflow_dispatch` kept so it can still be run by hand if ever needed. |
| `flutter-tag.yml` | `workflow_dispatch`, `push` (tags matching `v?\d+.\d+.\d+(-\d+)?`) | Builds+publishes a tagged release via `flutter-build.yml`. | UPSTREAM LEGACY | Only if a matching tag is pushed. ENZU does not push version tags as part of this workflow, so effectively dormant unless someone tags a commit. |
| `fdroid.yml` | `workflow_dispatch`, `push` (same tag pattern as above) | Publishes the F-Droid updater version file on tagged releases. | UPSTREAM LEGACY | Same as `flutter-tag.yml` — tag-push only. |
| `playground.yml` | `workflow_dispatch` (a `schedule` trigger exists in the file but is **already commented out** upstream) | Ad-hoc/experimental multi-platform build ("playground"). | MANUAL UTILITY | No — its cron is already disabled upstream; only `workflow_dispatch` is live. |
| `clear-cache.yml` | `workflow_dispatch` | Clears GitHub Actions caches for the repo. | MANUAL UTILITY | No. |
| `third-party-RustDeskTempTopMostWindow.yml` | `workflow_call` only | Reusable: builds a third-party helper DLL. Not directly triggerable, not part of the ENZU build. | UPSTREAM LEGACY (reusable) | No — only runs when called by another workflow. |

## What changed this phase

- **`flutter-nightly.yml`**: commented out the `schedule: cron: "0 0 * * *"` trigger,
  kept `workflow_dispatch`. This is the only automatic **cron/scheduled** trigger found
  outside `enzu-build.yml`'s own scope (`playground.yml`'s cron was already disabled
  upstream). This directly satisfies the task's explicit requirement: *"Flutter Nightly
  Build must NOT continue automatically running on this ENZU branch."*
- Nothing else was changed. `ci.yml` / `flutter-ci.yml` (`pull_request`/`push` to
  `master`) and `flutter-tag.yml` / `fdroid.yml` (tag push) are **not** cron/scheduled
  triggers — the task's instruction to remove/disable was scoped to "unwanted automatic
  cron/scheduled execution," not to push/PR triggers. They are left as upstream
  provides them; see "Not changed, and why" below.

## Not changed, and why

- **`ci.yml` / `flutter-ci.yml`** only fire on PRs/pushes to `master` (or manual
  dispatch). This fork currently develops on `feature/enzu-custom-client`, so they are
  dormant in day-to-day ENZU work and only become relevant at the point this branch is
  merged into (or replaces) `master` — a decision for a later phase, not this
  structural migration. Disabling them now would also touch upstream files
  (`ci.yml`, `flutter-ci.yml`) that the migration's safety rule asks to leave alone
  unless there's a demonstrated need.
- **`flutter-tag.yml` / `fdroid.yml`** only fire on a version-tag push. ENZU's own
  release process (artifacts via `enzu-build.yml`, manual dispatch) doesn't push tags
  matching that pattern, so these are already inert in practice; no edit needed.
- **`bridge.yml` / `flutter-build.yml` / `third-party-RustDeskTempTopMostWindow.yml`**
  are `workflow_call`-only reusable workflows — they cannot run on their own and cost
  nothing unless something calls them. `flutter-nightly.yml` was the only thing calling
  `flutter-build.yml` on a schedule; with that cron disabled, none of these three fire
  automatically anymore either.
- Per the task's explicit instruction, no workflow file was deleted, and
  `.github/workflows/enzu-build.yml` (and the `ci/register-enzu-build-workflow`
  process, if any, that manages it) was not touched at all.

## Non-GitHub CI (for completeness)

`.gitlab-ci.yml` at the repo root is a second, ENZU-owned, additive CI pipeline
(GitLab: Android arm64-v8a only, via the `docker/builder/` image). It is documented in
[docs/ENZU_BUILD_SYSTEM.md](../ENZU_BUILD_SYSTEM.md) and out of scope for this
GitHub-workflow audit; it has no automatic schedule trigger to disable.
