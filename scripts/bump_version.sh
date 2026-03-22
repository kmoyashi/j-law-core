#!/usr/bin/env bash

# 全公開パッケージのバージョンを一括更新する。
#
# 使い方:
#   ./scripts/bump_version.sh <new-version>
#
# new-version は semver 形式 (例: 0.1.0) で指定する。
# 先頭に "v" を付けた場合は自動で除去する。

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <new-version>" >&2
  exit 1
fi

NEW_VERSION="${1#v}"

if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "error: version must be in semver format (e.g. 0.1.0)" >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

python3 - "$ROOT_DIR" "$NEW_VERSION" <<'PY'
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
new_version = sys.argv[2]

version_files = {
    "j-law-core": root / "crates/j-law-core/Cargo.toml",
    "j-law-registry": root / "crates/j-law-registry/Cargo.toml",
    "j-law-c-ffi": root / "crates/j-law-c-ffi/Cargo.toml",
    "j-law-wasm": root / "crates/j-law-wasm/Cargo.toml",
    "j-law-python": root / "crates/j-law-python/pyproject.toml",
    "j_law_ruby": root / "crates/j-law-ruby/j_law_ruby.gemspec",
}


def bump_version(name: str, path: Path, new_ver: str) -> None:
    text = path.read_text(encoding="utf-8")

    if path.name == "j_law_ruby.gemspec":
        pattern = r'^(\s*spec\.version\s*=\s*")[^"]+(")\s*$'
    else:
        pattern = r'^(\s*version\s*=\s*")[^"]+(")\s*$'

    new_text, count = re.subn(pattern, rf"\g<1>{new_ver}\2", text, count=1, flags=re.MULTILINE)
    if count == 0:
        raise SystemExit(f"failed to find version in {path}")

    path.write_text(new_text, encoding="utf-8")
    print(f"  {name}: {path.relative_to(root)} -> {new_ver}")


print(f"Bumping all packages to {new_version}")
for name, path in version_files.items():
    bump_version(name, path, new_version)

print("done")
PY
