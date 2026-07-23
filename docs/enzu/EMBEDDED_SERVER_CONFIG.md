# ENZU Embedded Server Configuration

How the ENZU RustDesk client ships with working server defaults so a fresh install needs
no manual Settings step.

## 1. ID Server

`remote.enzutech.de` (hbbs, rendezvous). Option key: `custom-rendezvous-server`.

## 2. Relay Server

`relay.enzutech.de` (hbbr). Option key: `relay-server`.

## 3. Public key

- The committed value is the RustDesk **server public key** (contents of `id_ed25519.pub`).
  A public key is **not** a secret; committing it to source is intentional and safe.
- **SHA-256 fingerprint of the decoded 32-byte key:**
  `9df7628517123f41de4b54b8195042082ddd2564a84eec1040f89e90c677ad24`
- Validation: exactly one line, valid base64, decodes to 32 bytes. Verified by
  `scripts/verify_enzu_config.py` and the `enzu_config` unit tests.
- **No private-key material is present** anywhere in the repo. `.gitignore` blocks
  `id_ed25519` / `**/id_ed25519`.

## 4. Source of truth

**`src/enzu_config.rs`** — the only place the hostnames, key, config version, and policy
are defined. `verify_enzu_config.py` fails the build if these values appear in any other
tracked *code* file (docs excluded).

## 5. Exact startup hook

`apply_enzu_defaults()` is called from two minimal, idempotent hooks:

- `src/core_main.rs:35` — every desktop process (incl. the Windows `--server` subprocess
  that registers with hbbs).
- `src/flutter_ffi.rs` `initialize()` — Android and Flutter desktop, before the first
  rendezvous lookup.

A `std::sync::Once` guard makes the body run at most once per process even if both hooks
fire (desktop Flutter).

## 6. DEFAULT_SETTINGS resolution behavior

`apply_enzu_defaults()` writes the three options into `DEFAULT_SETTINGS`. `Config::get_option`
resolves in priority order (`libs/hbb_common/src/config.rs:1245`):

```
OVERWRITE_SETTINGS  >  user-saved options (RustDesk2.toml)  >  DEFAULT_SETTINGS
```

ENZU uses the **lowest** tier, so it only fills in a value when the user has not set one.

## 7. Why existing user settings are not overwritten

Because ENZU writes `DEFAULT_SETTINGS` only. A non-empty user-saved value sits in a
higher-priority tier and always wins. ENZU never calls `set_option` for these keys and
never writes `OVERWRITE_SETTINGS` (that would be the future `enforced` policy).

## 8. Fresh-install behavior

No user config exists → `get_option` falls through to `DEFAULT_SETTINGS` → the client uses
`remote.enzutech.de` / `relay.enzutech.de` / the ENZU key immediately, with no Settings step.

## 9. Upgrade behavior

Installing a newer ENZU build over an older one keeps the user's `RustDesk2.toml`. Any
value the user explicitly set is preserved (still wins); anything unset continues to use
the (possibly updated) ENZU defaults. `ENZU_CONFIG_VERSION` is available to gate future
migrations.

## 10. Clean-reinstall behavior

Uninstall including app data → no `RustDesk2.toml` → behaves exactly like a fresh install
(ENZU defaults active).

## 11. Offline-first startup behavior

`apply_enzu_defaults()` runs before any network activity, purely in memory. So even with no
connectivity at first launch, the ENZU servers are already the effective configuration and
are used as soon as the network is available. No config file is written to install defaults.

## 12. How to update hostnames

Edit `ENZU_ID_SERVER` / `ENZU_RELAY_SERVER` in `src/enzu_config.rs`, then run
`python scripts/verify_enzu_config.py`. Update the expected values in the verifier
(`EXPECTED_ID_SERVER` / `EXPECTED_RELAY_SERVER`) if the canonical hostnames change.

## 13. How to rotate the RustDesk server key safely

1. On the hbbs host, rotate the key pair; capture the **new** `id_ed25519.pub`.
2. Validate locally: base64, 32 bytes (the verifier does this).
3. Replace `ENZU_PUBLIC_KEY` in `src/enzu_config.rs`; update the fingerprint comment and
   this doc's fingerprint.
4. Run `verify_enzu_config.py`; bump `ENZU_CONFIG_VERSION` if you need migration logic.
5. Because clients read `key` from `DEFAULT_SETTINGS`, a user who *manually* set an old key
   will keep it (higher tier). Communicate rotations; consider the future `enforced`
   policy if you must guarantee propagation.
6. **Never** commit `id_ed25519` (private). Only the `.pub` value, as a constant.

## 14. Rollback procedure

- Revert the commit (`git revert <sha>`), or restore the previous `ENZU_PUBLIC_KEY` /
  hostnames in `src/enzu_config.rs`. No persisted client state depends on the change
  (defaults are in-memory), so rollback is clean. Users who never overrode settings pick up
  the reverted defaults on next launch.

## 15. How GitHub and GitLab verify the values

- **GitHub** (`.github/workflows/enzu-build.yml`): a dedicated `verify-config` job runs
  `scripts/verify_enzu_config.py`; `android-arm64` and `windows-x64` depend on it.
- **GitLab** (`.gitlab-ci.yml`): the `android-arm64` job runs the same script first.
- Neither CI stores the server values in YAML — both compile `src/enzu_config.rs`, so the
  values cannot drift between systems.

## 16. How to test relay fallback

Force a network where direct P2P is impossible (e.g. both peers behind strict/symmetric
NAT, or block UDP hole-punching), then connect. Confirm relay use via non-sensitive client
logs (relay negotiation) or hbbr logs on `relay.enzutech.de`. Note that relay is a
fallback — a directly reachable pair will not use it. See `docs/architecture/NETWORK_FLOW.md`.

## 17. Future default / enforced / hidden roadmap

- **default** (this phase): first-run defaults, user-overridable, never restored.
- **enforced** (future): also write `OVERWRITE_SETTINGS` so ENZU values always win.
- **hidden** (future): disable/remove the network settings UI in `flutter/lib`.

`EnzuServerPolicy` already models all three; only `Default` is wired. See
`docs/architecture/CUSTOMIZATION_GUIDE.md`.

## Security note

Embedding the public key and server addresses only **selects** the ENZU infrastructure. It
does not disable encryption, bypass key verification, weaken authorization, enable
unattended access, set a password, change screen-capture/Accessibility behavior, open
ports, or modify hbbs/hbbr. See the security review in the phase report.
