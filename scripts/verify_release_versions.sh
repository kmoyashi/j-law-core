#!/usr/bin/env bash

set -euo pipefail

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [release-tag]" >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPECTED_VERSION=""

if [[ $# -eq 1 ]]; then
  EXPECTED_VERSION="${1#v}"
fi

python3 - "$ROOT_DIR" "$EXPECTED_VERSION" <<'PY'
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
expected = sys.argv[2]

version_files = {
    "j-law-core": root / "crates/j-law-core/Cargo.toml",
    "j-law-registry": root / "crates/j-law-registry/Cargo.toml",
    "j-law-c-ffi": root / "crates/j-law-c-ffi/Cargo.toml",
    "j-law-wasm": root / "crates/j-law-wasm/Cargo.toml",
    "j-law-python": root / "crates/j-law-python/pyproject.toml",
    "j_law_ruby": root / "crates/j-law-ruby/j_law_ruby.gemspec",
}


def extract_version(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    if path.name == "j_law_ruby.gemspec":
        pattern = r'^\s*spec\.version\s*=\s*"([^"]+)"\s*$'
    else:
        pattern = r'^\s*version\s*=\s*"([^"]+)"\s*$'

    match = re.search(pattern, text, re.MULTILINE)
    if match is None:
        raise SystemExit(f"failed to find version in {path}")
    return match.group(1)


versions = {name: extract_version(path) for name, path in version_files.items()}
unique_versions = sorted(set(versions.values()))

for name, version in versions.items():
    print(f"{name}: {version}")

if len(unique_versions) != 1:
    joined = ", ".join(f"{name}={version}" for name, version in versions.items())
    raise SystemExit(f"version mismatch detected: {joined}")

actual = unique_versions[0]
if expected and actual != expected:
    raise SystemExit(f"release tag version {expected} does not match manifest version {actual}")

if expected:
    print(f"release tag matches manifest version: {actual}")
else:
    print(f"all manifest versions are aligned: {actual}")
PY
