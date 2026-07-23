#!/usr/bin/env python3
"""Deterministic verifier for the ENZU build-time server configuration.

Values are injected at build time (build.rs) with precedence:
    CI env var  >  repo-root .env (local dev)  >  src/enzu_config.rs FALLBACK_* constants
and read in Rust via `option_env!`. This script mirrors that resolution so it can
report and validate the EFFECTIVE configuration a build would bake in, plus assert
the committed fallback constants remain the stock ENZU deployment and are not
duplicated in other tracked code.

Prints only safe information (hostnames, config version, policy, per-value source,
public-key SHA-256 fingerprint). Never prints or requires the private key.
Exit 0 on success, non-zero with a clear reason on any failure.
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

EXPECTED_FALLBACK_ID = "remote.enzutech.de"
EXPECTED_FALLBACK_RELAY = "relay.enzutech.de"
EXPECTED_POLICY = "default"

UPSTREAM_RS_PUB_KEY = "OeVuKk5nlHiXp+APNn0Y3pC1Iwpwn44JGqrQCsWqmBw="
FORBIDDEN_HOST_SUBSTRINGS = ("rustdesk.com", "localhost", "127.0.0.1", "0.0.0.0")
PLACEHOLDER_MARKERS = ("<", ">", "PLACEHOLDER", "REPLACE", "CHANGEME", "TODO", "XXXX", "EXAMPLE")

ENV_KEYS = ("ENZU_ID_SERVER", "ENZU_RELAY_SERVER", "ENZU_PUBLIC_KEY")
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


def extract_const(src: str, name: str) -> str:
    m = re.search(rf'const {name}: &str = "([^"]*)";', src)
    if not m:
        fail(f"could not find `{name}` in {SOURCE_OF_TRUTH}")
    return m.group(1)


def require_option_env(src: str) -> None:
    for key in ENV_KEYS:
        if f'option_env!("{key}")' not in src:
            fail(f"{SOURCE_OF_TRUTH} does not read `{key}` via option_env! (build-time injection missing)")


def read_dotenv() -> dict:
    env = {}
    path = os.path.join(REPO_ROOT, ".env")
    if os.path.isfile(path):
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                line = line.strip()
                if not line or line.startswith("#") or "=" not in line:
                    continue
                k, v = line.split("=", 1)
                v = v.strip()
                if len(v) >= 2 and ((v[0] == v[-1] == '"') or (v[0] == v[-1] == "'")):
                    v = v[1:-1]
                env[k.strip()] = v
    return env


def contains_placeholder(value: str) -> bool:
    up = value.upper()
    return any(m in up for m in PLACEHOLDER_MARKERS)


def validate_host(label: str, value: str) -> None:
    if not value:
        fail(f"{label} is empty")
    if contains_placeholder(value):
        fail(f"{label} contains placeholder text: {value!r}")
    low = value.lower()
    for bad in FORBIDDEN_HOST_SUBSTRINGS:
        if bad in low:
            fail(f"{label} points at forbidden/public infra ({bad!r}): {value!r}")


def validate_key(value: str) -> bytes:
    if not value:
        fail("public key is empty")
    if contains_placeholder(value):
        fail(f"public key contains placeholder text: {value!r}")
    if value == UPSTREAM_RS_PUB_KEY:
        fail("public key equals the UPSTREAM RustDesk key (public infra), not ENZU")
    if value.strip() != value or " " in value or "\n" in value:
        fail("public key must be exactly one whitespace-free line")
    try:
        raw = base64.b64decode(value, validate=True)
    except (binascii.Error, ValueError) as exc:
        fail(f"public key is not valid base64: {exc}")
    if len(raw) != 32:
        fail(f"decoded public key is {len(raw)} bytes, expected 32")
    return raw


def resolve(key: str, dotenv: dict, fallback: str) -> tuple[str, str]:
    """Mirror build.rs precedence. A present-but-empty env/.env value is 'provided'
    and will fail validation; a completely absent value uses the fallback."""
    if key in os.environ:
        return os.environ[key].strip(), "CI env var"
    if key in dotenv:
        return dotenv[key].strip(), "local .env"
    return fallback, "built-in fallback"


def tracked_code_files() -> list:
    try:
        out = subprocess.run(["git", "ls-files"], cwd=REPO_ROOT, check=True,
                             capture_output=True, text=True).stdout
        files = out.splitlines()
    except (OSError, subprocess.CalledProcessError):
        files = []
        for base in ("src", "libs", "flutter/lib", "flutter/android"):
            for root, _d, names in os.walk(os.path.join(REPO_ROOT, base)):
                for n in names:
                    files.append(os.path.relpath(os.path.join(root, n), REPO_ROOT))
    return [f for f in files if f.lower().endswith(CODE_EXTS)]


def check_single_source_of_truth(values: list) -> None:
    sot = SOURCE_OF_TRUTH.replace("\\", "/")
    this_script = os.path.relpath(os.path.abspath(__file__), REPO_ROOT).replace("\\", "/")
    needles = [v for v in values if v]
    offenders = []
    for rel in tracked_code_files():
        relf = rel.replace("\\", "/")
        if relf in (sot, this_script):
            continue
        try:
            with open(os.path.join(REPO_ROOT, rel), encoding="utf-8", errors="ignore") as fh:
                text = fh.read()
        except OSError:
            continue
        if any(n in text for n in needles):
            offenders.append(relf)
    if offenders:
        fail("ENZU deployment values duplicated outside the source of truth "
             f"(keep them only in {SOURCE_OF_TRUTH} / .env / CI): {', '.join(sorted(offenders))}")


def main() -> None:
    src = read_source()
    require_option_env(src)

    # 1) Committed fallback constants must remain the stock ENZU deployment.
    fb_id = extract_const(src, "FALLBACK_ID_SERVER")
    fb_relay = extract_const(src, "FALLBACK_RELAY_SERVER")
    fb_key = extract_const(src, "FALLBACK_PUBLIC_KEY")
    validate_host("FALLBACK_ID_SERVER", fb_id)
    validate_host("FALLBACK_RELAY_SERVER", fb_relay)
    validate_key(fb_key)
    if fb_id != EXPECTED_FALLBACK_ID:
        fail(f"FALLBACK_ID_SERVER is {fb_id!r}, expected {EXPECTED_FALLBACK_ID!r}")
    if fb_relay != EXPECTED_FALLBACK_RELAY:
        fail(f"FALLBACK_RELAY_SERVER is {fb_relay!r}, expected {EXPECTED_FALLBACK_RELAY!r}")

    # 2) Policy / version.
    mpol = re.search(r"EnzuServerPolicy::(\w+);", src)
    policy = mpol.group(1).lower() if mpol else "?"
    if policy != EXPECTED_POLICY:
        fail(f"policy is {policy!r}, expected {EXPECTED_POLICY!r}")
    mver = re.search(r"ENZU_CONFIG_VERSION: u32 = (\d+);", src)
    version = int(mver.group(1)) if mver else 0
    if version < 1:
        fail(f"config version must be >= 1, got {version}")

    # 3) Effective values (what THIS environment would bake in).
    dotenv = read_dotenv()
    eff_id, src_id = resolve("ENZU_ID_SERVER", dotenv, fb_id)
    eff_relay, src_relay = resolve("ENZU_RELAY_SERVER", dotenv, fb_relay)
    eff_key, src_key = resolve("ENZU_PUBLIC_KEY", dotenv, fb_key)
    validate_host("effective ID server", eff_id)
    validate_host("effective relay server", eff_relay)
    raw = validate_key(eff_key)

    # 4) Single source of truth for the committed (fallback) values.
    check_single_source_of_truth([fb_id, fb_relay, fb_key])

    fingerprint = hashlib.sha256(raw).hexdigest()
    print("[ENZU-VERIFY] OK")
    print(f"  config version : {version}")
    print(f"  policy         : {policy}")
    print(f"  id server      : {eff_id}   (source: {src_id})")
    print(f"  relay server   : {eff_relay}   (source: {src_relay})")
    print(f"  public key     : valid base64, 32 bytes   (source: {src_key})")
    print(f"  key SHA-256    : {fingerprint}")


if __name__ == "__main__":
    main()
