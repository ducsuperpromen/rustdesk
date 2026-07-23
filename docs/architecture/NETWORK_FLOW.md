# Network Flow

The effective sequence from app start to a connected session. Ports are RustDesk
defaults (`RENDEZVOUS_PORT 21116`, `RELAY_PORT 21117`, `libs/hbb_common/src/config.rs`).

```
app start
  └─ apply_enzu_defaults()                      # writes DEFAULT_SETTINGS (in-memory)
       custom-rendezvous-server = remote.enzutech.de
       relay-server             = relay.enzutech.de
       key                      = <ENZU public key>
  └─ resolve effective settings                 # Config::get_option precedence:
       OVERWRITE_SETTINGS > user-saved > DEFAULT_SETTINGS
       (a user/admin value, if present, wins over the ENZU default)
  └─ contact hbbs at remote.enzutech.de:21116   # rendezvous_mediator
  └─ register this device's RustDesk ID
  └─ (to connect to a peer)
       ├─ attempt DIRECT / NAT hole-punching     # preferred path
       └─ if direct fails -> RELAY fallback via relay.enzutech.de:21117 (hbbr)
  └─ verify the server using the embedded public key
       # RustDesk end-to-end encryption & key verification unchanged
```

Notes:

- **Not every session traverses the relay.** The relay is a *fallback* used only when a
  direct/NAT-traversed peer-to-peer path cannot be established. Many sessions connect
  directly.
- **First run is offline-safe.** `apply_enzu_defaults()` runs before any network call, so
  the ENZU servers are already the effective configuration once connectivity is available
  — no manual Settings step, no first-connection race.
- **Key role.** The embedded value is the *server* public key (from `id_ed25519.pub`). It
  selects/authenticates the ENZU infrastructure. It is **not** a device password, not
  unattended-access, and does not weaken RustDesk's encryption or the incoming-connection
  authorization prompts.
- **ID vs Relay vs key** are distinct: the device's own RustDesk ID is separate from the
  ID *server*; the relay server is separate from the ID server; the public key is not a
  private key and not an API server (ENZU runs OSS, no API server configured).
