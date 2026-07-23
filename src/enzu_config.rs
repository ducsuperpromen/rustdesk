//! ENZU embedded server configuration — build-time injected, one place.
//!
//! A fresh install connects to the ENZU infrastructure with no manual Settings
//! step. The concrete server values are **injected at build time** so the same
//! source builds ENZU, DEV, TEST, or a customer environment without edits.
//!
//! Value precedence (resolved in `build.rs`, exported via `cargo:rustc-env`, read
//! here with `option_env!`):
//!   1. CI build environment variables (GitHub Actions Variables / GitLab CI vars)
//!   2. a local `.env` at the repo root (developer convenience; git-ignored)
//!   3. the built-in `FALLBACK_*` constants below (stock ENZU deployment)
//!
//! `build.rs` **fails the build** if a value is *provided but invalid* (empty,
//! placeholder, bad hostname, localhost, a public RustDesk server, or a public key
//! that is not 32 bytes). A value that is *completely absent* falls back to the
//! constants below — so upstream/offline `cargo build` still works and never
//! silently selects the public RustDesk infrastructure. See
//! `docs/enzu/EMBEDDED_SERVER_CONFIG.md`.
//!
//! Runtime behaviour is unchanged from the previous phase: values are applied into
//! RustDesk's `DEFAULT_SETTINGS` (the LOWEST-priority tier of `Config::get_option`,
//! `OVERWRITE_SETTINGS` > user-saved options > `DEFAULT_SETTINGS`), in-memory only,
//! so user/administrator changes always win and nothing is written on startup.

use hbb_common::config::{keys, DEFAULT_SETTINGS};
use std::collections::HashMap;
use std::sync::Once;

/// Schema version for the embedded configuration. Bump when the meaning or set of
/// embedded values changes, so future migrations can reason about older builds.
pub const ENZU_CONFIG_VERSION: u32 = 1;

/// Server-configuration policy. Only [`EnzuServerPolicy::Default`] is implemented
/// in this phase; the other variants are reserved so future work is a small,
/// localized change rather than a redesign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnzuServerPolicy {
    /// Apply ENZU values on first run, but allow later user/admin changes.
    Default,
    /// (Future) Restore ENZU values if changed. NOT implemented in this phase.
    Enforced,
    /// (Future) Hide/disable the network settings UI. NOT implemented in this phase.
    Hidden,
}

impl EnzuServerPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            EnzuServerPolicy::Default => "default",
            EnzuServerPolicy::Enforced => "enforced",
            EnzuServerPolicy::Hidden => "hidden",
        }
    }
}

/// Active policy for this build. This phase ships only `default`.
pub const ENZU_SERVER_POLICY: EnzuServerPolicy = EnzuServerPolicy::Default;

// ---------------------------------------------------------------------------
// Fallback constants — the stock ENZU deployment. These are used ONLY when no
// build-time value is injected (no CI variable and no `.env`). They are the single
// place ENZU deployment values live in committed source; `build.rs` and
// `scripts/verify_enzu_config.py` treat this file as the source of truth.
// A public key is not a secret, so committing it here is intentional.
// ---------------------------------------------------------------------------

/// Fallback ENZU ID / rendezvous (hbbs) server.
const FALLBACK_ID_SERVER: &str = "remote.enzutech.de";
/// Fallback ENZU relay (hbbr) server.
const FALLBACK_RELAY_SERVER: &str = "relay.enzutech.de";
/// Fallback ENZU RustDesk server PUBLIC key (contents of `id_ed25519.pub`, base64,
/// 32 bytes). SHA-256 of the decoded bytes:
/// 9df7628517123f41de4b54b8195042082ddd2564a84eec1040f89e90c677ad24
const FALLBACK_PUBLIC_KEY: &str = "5d+iJ5z0+NuD377LgB1CBvlz4KQqbet3QFW+gCe4h3k=";

/// Effective ENZU ID / rendezvous (hbbs) server (build-time value or fallback).
pub const ENZU_ID_SERVER: &str = match option_env!("ENZU_ID_SERVER") {
    Some(v) => v,
    None => FALLBACK_ID_SERVER,
};

/// Effective ENZU relay (hbbr) server (build-time value or fallback).
pub const ENZU_RELAY_SERVER: &str = match option_env!("ENZU_RELAY_SERVER") {
    Some(v) => v,
    None => FALLBACK_RELAY_SERVER,
};

/// Effective ENZU RustDesk server PUBLIC key (build-time value or fallback).
pub const ENZU_PUBLIC_KEY: &str = match option_env!("ENZU_PUBLIC_KEY") {
    Some(v) => v,
    None => FALLBACK_PUBLIC_KEY,
};

static APPLY_ONCE: Once = Once::new();

/// Insert the ENZU defaults into a settings map, keyed by the exact upstream
/// option names resolved by `Config::get_option`. Pure and idempotent so it can
/// be unit-tested without touching global state.
fn insert_enzu_defaults(map: &mut HashMap<String, String>) {
    map.insert(
        keys::OPTION_CUSTOM_RENDEZVOUS_SERVER.to_string(),
        ENZU_ID_SERVER.to_string(),
    );
    map.insert(
        keys::OPTION_RELAY_SERVER.to_string(),
        ENZU_RELAY_SERVER.to_string(),
    );
    map.insert(keys::OPTION_KEY.to_string(), ENZU_PUBLIC_KEY.to_string());
}

/// Apply the ENZU defaults into RustDesk's `DEFAULT_SETTINGS` (policy: default).
///
/// Idempotent and cheap: runs its body once per process (guarded by [`Once`]) and
/// only writes the in-memory `DEFAULT_SETTINGS` map — never a config file, never
/// `set_option`, never `OVERWRITE_SETTINGS`. Because `DEFAULT_SETTINGS` is the
/// lowest-priority tier, any value the user/administrator has saved still wins.
///
/// Call this at the earliest client startup point, before the first rendezvous /
/// server-configuration lookup.
pub fn apply_enzu_defaults() {
    APPLY_ONCE.call_once(|| {
        {
            let mut map = DEFAULT_SETTINGS.write().unwrap();
            insert_enzu_defaults(&mut map);
        }
        // Do NOT log the key (even though it is public) to keep logs clean.
        hbb_common::log::info!(
            "ENZU defaults applied (config v{}, policy={}): id_server={}, relay_server={}",
            ENZU_CONFIG_VERSION,
            ENZU_SERVER_POLICY.as_str(),
            ENZU_ID_SERVER,
            ENZU_RELAY_SERVER,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use hbb_common::base64::{engine::general_purpose::STANDARD, Engine as _};
    use hbb_common::config::{Config, DEFAULT_SETTINGS, OVERWRITE_SETTINGS};

    #[test]
    fn policy_and_version_are_expected() {
        assert_eq!(ENZU_CONFIG_VERSION, 1);
        assert_eq!(ENZU_SERVER_POLICY, EnzuServerPolicy::Default);
        assert_eq!(ENZU_SERVER_POLICY.as_str(), "default");
    }

    #[test]
    fn fallback_constants_are_the_stock_enzu_deployment() {
        assert_eq!(FALLBACK_ID_SERVER, "remote.enzutech.de");
        assert_eq!(FALLBACK_RELAY_SERVER, "relay.enzutech.de");
        let raw = STANDARD
            .decode(FALLBACK_PUBLIC_KEY)
            .expect("FALLBACK_PUBLIC_KEY must be valid base64");
        assert_eq!(raw.len(), 32, "fallback public key must be 32 bytes");
    }

    #[test]
    fn effective_values_are_non_empty_and_key_is_valid() {
        // Whatever was injected at build time (or the fallback) must be usable.
        assert!(!ENZU_ID_SERVER.is_empty());
        assert!(!ENZU_RELAY_SERVER.is_empty());
        let raw = STANDARD
            .decode(ENZU_PUBLIC_KEY)
            .expect("ENZU_PUBLIC_KEY must be valid base64");
        assert_eq!(
            raw.len(),
            32,
            "effective public key must decode to 32 bytes"
        );
        // Same validation path the client itself uses to build a verifying key.
        assert!(
            crate::common::get_rs_pk(ENZU_PUBLIC_KEY).is_some(),
            "get_rs_pk must accept the effective ENZU public key"
        );
    }

    #[test]
    fn insert_enzu_defaults_is_pure_and_idempotent() {
        let mut m = HashMap::new();
        insert_enzu_defaults(&mut m);
        let after_first = m.clone();
        insert_enzu_defaults(&mut m); // second call must not change anything
        assert_eq!(m, after_first, "insert_enzu_defaults must be idempotent");
        assert_eq!(m.len(), 3);
        assert_eq!(
            m.get(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER)
                .map(String::as_str),
            Some(ENZU_ID_SERVER)
        );
        assert_eq!(
            m.get(keys::OPTION_RELAY_SERVER).map(String::as_str),
            Some(ENZU_RELAY_SERVER)
        );
        assert_eq!(
            m.get(keys::OPTION_KEY).map(String::as_str),
            Some(ENZU_PUBLIC_KEY)
        );
    }

    // Touches the global in-memory settings maps. Self-contained: it saves and
    // restores the maps and uses a unique probe key so it does not depend on (or
    // pollute) any real user config, and writes nothing to disk.
    #[test]
    fn default_settings_resolution_and_priority() {
        let saved_default = DEFAULT_SETTINGS.read().unwrap().clone();
        let saved_overwrite = OVERWRITE_SETTINGS.read().unwrap().clone();

        // 1) Defaults land in DEFAULT_SETTINGS.
        {
            let mut m = DEFAULT_SETTINGS.write().unwrap();
            insert_enzu_defaults(&mut m);
        }
        assert_eq!(
            DEFAULT_SETTINGS
                .read()
                .unwrap()
                .get(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER)
                .map(String::as_str),
            Some(ENZU_ID_SERVER)
        );
        assert_eq!(
            DEFAULT_SETTINGS
                .read()
                .unwrap()
                .get(keys::OPTION_KEY)
                .map(String::as_str),
            Some(ENZU_PUBLIC_KEY)
        );

        // 2) A clean lookup falls back to DEFAULT_SETTINGS (unique key => no
        //    user-saved value can exist for it).
        let probe = "enzu-test-probe-key";
        OVERWRITE_SETTINGS.write().unwrap().remove(probe);
        DEFAULT_SETTINGS
            .write()
            .unwrap()
            .insert(probe.to_string(), "default-value".to_string());
        assert_eq!(Config::get_option(probe), "default-value");

        // 3) OVERWRITE_SETTINGS has the highest priority.
        OVERWRITE_SETTINGS
            .write()
            .unwrap()
            .insert(probe.to_string(), "overwrite-value".to_string());
        assert_eq!(Config::get_option(probe), "overwrite-value");

        // Restore global state.
        *DEFAULT_SETTINGS.write().unwrap() = saved_default;
        *OVERWRITE_SETTINGS.write().unwrap() = saved_overwrite;
    }
}
