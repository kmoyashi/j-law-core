#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
GO_MODULE_DIR="${WORKSPACE_ROOT}/crates/j-law-go"
DEST_ROOT="${JLAW_GO_NATIVE_DEST_ROOT:-${GO_MODULE_DIR}/native}"
TARGET_TRIPLE="${JLAW_GO_NATIVE_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}"

case "${TARGET_TRIPLE}" in
  aarch64-apple-darwin)
    PLATFORM_DIR="darwin_arm64"
    ;;
  x86_64-apple-darwin)
    PLATFORM_DIR="darwin_amd64"
    ;;
  x86_64-unknown-linux-gnu)
    PLATFORM_DIR="linux_amd64"
    ;;
  aarch64-unknown-linux-gnu)
    PLATFORM_DIR="linux_arm64"
    ;;
  *)
    echo "unsupported target triple for j-law-go native archive sync: ${TARGET_TRIPLE}" >&2
    exit 1
    ;;
esac

cargo build \
  -p j-law-c-ffi \
  --release \
  --manifest-path "${WORKSPACE_ROOT}/Cargo.toml" \
  --target "${TARGET_TRIPLE}"

SOURCE_ARCHIVE="${WORKSPACE_ROOT}/target/${TARGET_TRIPLE}/release/libj_law_c_ffi.a"
DEST_DIR="${DEST_ROOT}/${PLATFORM_DIR}"
DEST_ARCHIVE="${DEST_DIR}/libj_law_c_ffi.a"

mkdir -p "${DEST_DIR}"
cp "${SOURCE_ARCHIVE}" "${DEST_ARCHIVE}"

echo "synced ${DEST_ARCHIVE}"
