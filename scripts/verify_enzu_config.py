#!/usr/bin/env python3
"""Deterministic verifier for the ENZU embedded server configuration.

Single source of truth: src/enzu_config.rs. This script parses that file, validates
the embedded values, and enforces that the ENZU hostnames / public key are not
duplicated in other tracked *code* files. It prints only safe information
(hostnames, config version, policy, public-key SHA-256 fingerprint) and never the
private key (which does not exist here).

Exit code 0 on success, non-zero (with a clear reason) on any failure. CI runs this
before the platform builds so a misconfiguration fails fast.
"""
from __future__ import annotations

import base64
import binascii
import hashlib
import os
import re
import subprocess
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCE_OF_TRUTH = os.path.join("src", "enzu_config.rs")

EXPECTED_ID_SERVER = "remote.enzutech.de"
EXPECTED_RELAY_SERVER = "relay.enzutech.de"
EXPECTED_POLICY = "default"

# Public RustDesk infrastructure / upstream default key that must never be used as
# an ENZU value or fallback.
UPSTREAM_RS_PUB_KEY = "OeVuKk5nlHiXp+APNn0Y3pC1Iwpwn44JGqrQCsWqmBw="
FORBIDDEN_HOST_SUBSTRINGS = ("rustdesk.com", "localhost", "127.0.0.1", "0.0.0.0")
PLACEHOLDER_MARKERS = ("<", ">", "PLACEHOLDER", "REPLACE", "CHANGEME", "TODO", "XXXX", "EXAMPLE")

# Files allowed to legitimately contain the ENZU values (source of truth + docs +
# this verifier). Everything else that is *code* must not duplicate them.
CODE_EXTS = (".rs", ".dart", ".kt", ".java", ".gradle", ".toml", ".yml", ".yaml",
             ".cpp", ".cc", ".c", ".h", ".hpp", ".py", ".sh", ".ps1")


def fail(msg: str) -> "NoReturn":  # type: ignore[name-defined]
    print(f"[ENZU-VERIFY] FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def read_source() -> str:
    path = os.path.join(REPO_ROOT, SOURCE_OF_TRUTH)
    if not os.path.isfile(path):
        fail(f"source of truth not found: {SOURCE_OF_TRUTH}")
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def extract_str(src: str, name: str) -> str:
    m = re.search(rf'pub const {name}: &str = "([^"]*)";', src)
    if not m:
        fail(f"could not find `{name}` in {SOURCE_OF_TRUTH}")
    return m.group(1)


def extract_version(src: str) -> int:
    m = re.search(r"pub const ENZU_CONFIG_VERSION: u32 = (\d+);", src)
    if not m:
        fail("could not find `ENZU_CONFIG_VERSION`")
    return int(m.group(1))


def extract_policy(src: str) -> str:
    m = re.search(r"pub const ENZU_SERVER_POLICY: EnzuServerPolicy = EnzuServerPolicy::(\w+);", src)
    if not m:
        fail("could not find `ENZU_SERVER_POLICY`")
    return m.group(1).lower()


def contains_placeholder(value: str) -> bool:
    upper = value.upper()
    return any(marker in upper for marker in PLACEHOLDER_MARKERS)


def validate_host(label: str, value: str, expected: str) -> None:
    if not value:
        fail(f"{label} is empty")
    if contains_placeholder(value):
        fail(f"{label} contains placeholder text: {value!r}")
    low = value.lower()
    for bad in FORBIDDEN_HOST_SUBSTRINGS:
        if bad in low:
            fail(f"{label} points at forbidden/public infra ({bad!r}): {value!r}")
    if value != expected:
        fail(f"{label} is {value!r}, expected {expected!r}")


def validate_key(value: str) -> bytes:
    if not value:
        fail("public key is empty")
    if contains_placeholder(value):
        fail(f"public key contains placeholder text: {value!r}")
    if value == UPSTREAM_RS_PUB_KEY:
        fail("public key equals the UPSTREAM RustDesk key (public infra), not ENZU")
    if "\n" in value.strip() or " " in value.strip():
        fail("public key must be exactly one whitespace-free line")
    try:
        raw = base64.b64decode(value, validate=True)
    except (binascii.Error, ValueError) as exc:
        fail(f"public key is not valid base64: {exc}")
    if len(raw) != 32:
        fail(f"decoded public key is {len(raw)} bytes, expected 32")
    return raw


def tracked_code_files() -> list[str]:
    try:
        out = subprocess.run(
            ["git", "ls-files"], cwd=REPO_ROOT, check=True,
            capture_output=True, text=True,
        ).stdout
        files = out.splitlines()
    except (OSError, subprocess.CalledProcessError):
        # Fallback: walk likely code dirs if git is unavailable.
        files = []
        for base in ("src", "libs", "flutter/lib", "flutter/android"):
            for root, _dirs, names in os.walk(os.path.join(REPO_ROOT, base)):
                for n in names:
                    files.append(os.path.relpath(os.path.join(root, n), REPO_ROOT))
    return [f for f in files if f.lower().endswith(CODE_EXTS)]


def check_single_source_of_truth(id_server: str, key: str) -> None:
    """Fail if the ENZU hostname or key appears in any tracked code file other than
    the source of truth (docs are intentionally excluded)."""
    sot = SOURCE_OF_TRUTH.replace("\\", "/")
    this_script = os.path.relpath(os.path.abspath(__file__), REPO_ROOT).replace("\\", "/")
    offenders = []
    for rel in tracked_code_files():
        relf = rel.replace("\\", "/")
        if relf in (sot, this_script):
            continue
        path = os.path.join(REPO_ROOT, rel)
        try:
            with open(path, encoding="utf-8", errors="ignore") as fh:
                text = fh.read()
        except OSError:
            continue
        if id_server in text or key in text:
            offenders.append(relf)
    if offenders:
        fail("ENZU values duplicated outside the source of truth (move to "
             f"src/enzu_config.rs): {', '.join(sorted(offenders))}")


def main() -> None:
    src = read_source()
    version = extract_version(src)
    policy = extract_policy(src)
    id_server = extract_str(src, "ENZU_ID_SERVER")
    relay_server = extract_str(src, "ENZU_RELAY_SERVER")
    public_key = extract_str(src, "ENZU_PUBLIC_KEY")

    validate_host("ID server", id_server, EXPECTED_ID_SERVER)
    validate_host("relay server", relay_server, EXPECTED_RELAY_SERVER)
    raw = validate_key(public_key)

    if policy != EXPECTED_POLICY:
        fail(f"policy is {policy!r}, expected {EXPECTED_POLICY!r}")
    if version < 1:
        fail(f"config version must be >= 1, got {version}")

    check_single_source_of_truth(id_server, public_key)

    fingerprint = hashlib.sha256(raw).hexdigest()
    print("[ENZU-VERIFY] OK")
    print(f"  config version : {version}")
    print(f"  policy         : {policy}")
    print(f"  id server      : {id_server}")
    print(f"  relay server   : {relay_server}")
    print(f"  public key     : valid base64, 32 bytes")
    print(f"  key SHA-256    : {fingerprint}")


if __name__ == "__main__":
    main()
