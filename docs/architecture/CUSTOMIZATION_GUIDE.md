# Customization Guide

Safe customization points for the ENZU fork, and how to keep future upstream merges clean.
**This phase implements only the embedded server defaults.** Everything else below is
documented for later phases and is **not** changed now.

## Golden rules

- Prefer **new ENZU-owned files** over editing upstream files. Upstream never touches
  `enzu_config.rs`, `docker/builder/*`, `scripts/verify_enzu_config.py`, `enzu-*` — zero
  merge conflicts.
- When an upstream file must be touched, keep it to a **one-line call** (as done in
  `core_main.rs` and `flutter_ffi.rs`).
- Keep **one source of truth** per concern. `verify_enzu_config.py` enforces this for the
  server values.

## 1. Embedded server defaults (IMPLEMENTED)

- Source of truth: **`src/enzu_config.rs`** — `ENZU_ID_SERVER`, `ENZU_RELAY_SERVER`,
  `ENZU_PUBLIC_KEY`, `ENZU_CONFIG_VERSION`, `ENZU_SERVER_POLICY`.
- Applied into `DEFAULT_SETTINGS` by `apply_enzu_defaults()` (policy `default`).
- To change servers/key: edit only `enzu_config.rs`, then run `verify_enzu_config.py`.
- See `docs/enzu/EMBEDDED_SERVER_CONFIG.md`.

## 2. Server policy roadmap: default / enforced / hidden

`EnzuServerPolicy` already models all three; only `Default` is active.

- **default** (now): ENZU values on first run; user/admin can change them; never restored.
- **enforced** (future): also populate `OVERWRITE_SETTINGS` (highest priority) so ENZU
  values win even if a user changes them. Localized change in `enzu_config.rs`.
- **hidden** (future): additionally hide/disable the network settings UI in
  `flutter/lib` (Dart). This is a UI change, out of scope now.

## 3. Application name — NOT NOW

- Upstream uses `config::APP_NAME` (default `"RustDesk"`) and `read_custom_client` can set
  `app-name`. A future ENZU phase could set it centrally. Do not change in this phase.

## 4. Android package ID — NOT NOW

- `flutter/android/app/build.gradle` `applicationId "com.carriez.flutter_hbb"`. Changing it
  affects installs/signing/upgrades — deliberately deferred.

## 5. Icon / 6. Colors — NOT NOW

- Icons: `flutter/android/app/src/main/res/**`, `flutter/assets/**`.
- Colors/theme: `flutter/lib` theme. Branding phase only.

## 7. Versioning

- App `VERSION` (currently `1.4.9`) drives artifact names in CI. The ENZU config schema
  has its own `ENZU_CONFIG_VERSION` (independent of app version) for migration logic.

## 8. Android-specific branding — NOT NOW

- App label in `AndroidManifest.xml`, package ID, adaptive icons.

## 9. Windows-specific branding — NOT NOW

- Runner metadata/icon under `flutter/windows/runner/**`, MSI/portable naming in `build.py`.

## 10. Safe upstream-merge strategy

- Keep ENZU logic in `enzu_config.rs`; touch upstream files only via single calls.
- After merging `upstream/master`: re-run `scripts/verify_enzu_config.py` and the
  `enzu_config` unit tests; confirm the two one-line hooks still exist in `core_main.rs`
  and `flutter_ffi.rs`.
