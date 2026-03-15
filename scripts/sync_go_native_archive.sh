#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
GO_MODULE_DIR="${WORKSPACE_ROOT}/crates/j-law-go"
DEST_ROOT="${JLAW_GO_NATIVE_DEST_ROOT:-${GO_MODULE_DIR}/native}"
TARGET_TRIPLE="${JLAW_GO_NATIVE_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}"
ACTION="${JLAW_GO_NATIVE_ACTION:-sync}"

hash_file() {
  local path="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${path}" | awk '{print $1}'
  else
    echo "sha256 hash tool is not available" >&2
    exit 1
  fi
}

hash_stdin() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print $1}'
  else
    echo "sha256 hash tool is not available" >&2
    exit 1
  fi
}

collect_fingerprint_inputs() {
  {
    printf '%s\n' \
      "${WORKSPACE_ROOT}/Cargo.lock" \
      "${WORKSPACE_ROOT}/Cargo.toml" \
      "${WORKSPACE_ROOT}/crates/j-law-c-ffi/Cargo.toml" \
      "${WORKSPACE_ROOT}/crates/j-law-c-ffi/j_law_c_ffi.h" \
      "${WORKSPACE_ROOT}/crates/j-law-core/Cargo.toml" \
      "${WORKSPACE_ROOT}/crates/j-law-registry/Cargo.toml"
    find "${WORKSPACE_ROOT}/crates/j-law-c-ffi/src" -type f
    find "${WORKSPACE_ROOT}/crates/j-law-core/src" -type f
    find "${WORKSPACE_ROOT}/crates/j-law-registry/src" -type f
    find "${WORKSPACE_ROOT}/crates/j-law-registry/data" -type f
  } | LC_ALL=C sort
}

compute_source_fingerprint() {
  local entries=()
  local file

  while IFS= read -r file; do
    entries+=("$(hash_file "${file}")  ${file#"${WORKSPACE_ROOT}/"}")
  done < <(collect_fingerprint_inputs)

  {
    printf 'target=%s\n' "${TARGET_TRIPLE}"
    rustc -V
    printf '%s\n' "${entries[@]}"
  } | hash_stdin
}

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

DEST_DIR="${DEST_ROOT}/${PLATFORM_DIR}"
DEST_ARCHIVE="${DEST_DIR}/libj_law_c_ffi.a"
METADATA_FILE="${DEST_DIR}/build-info.txt"
SOURCE_FINGERPRINT="$(compute_source_fingerprint)"

write_metadata() {
  mkdir -p "${DEST_DIR}"
  cat > "${METADATA_FILE}" <<EOF
target_triple=${TARGET_TRIPLE}
rustc_version=$(rustc -V)
source_fingerprint=${SOURCE_FINGERPRINT}
EOF
}

if [[ "${ACTION}" == "verify" ]]; then
  if [[ ! -f "${DEST_ARCHIVE}" ]]; then
    echo "missing vendored native archive: ${DEST_ARCHIVE}" >&2
    exit 1
  fi

  if [[ ! -f "${METADATA_FILE}" ]]; then
    echo "missing vendored native metadata: ${METADATA_FILE}" >&2
    echo "Run 'make sync-native' and commit the generated metadata." >&2
    exit 1
  fi

  if ! grep -Fqx "target_triple=${TARGET_TRIPLE}" "${METADATA_FILE}" \
    || ! grep -Fqx "rustc_version=$(rustc -V)" "${METADATA_FILE}" \
    || ! grep -Fqx "source_fingerprint=${SOURCE_FINGERPRINT}" "${METADATA_FILE}"; then
    echo "vendored native archive is stale: ${DEST_ARCHIVE}" >&2
    echo "Run 'make sync-native' and commit the updated archive." >&2
    exit 1
  fi

  echo "verified ${DEST_ARCHIVE}"
  exit 0
fi

cargo build \
  -p j-law-c-ffi \
  --release \
  --manifest-path "${WORKSPACE_ROOT}/Cargo.toml" \
  --target "${TARGET_TRIPLE}"

SOURCE_ARCHIVE="${WORKSPACE_ROOT}/target/${TARGET_TRIPLE}/release/libj_law_c_ffi.a"
mkdir -p "${DEST_DIR}"
cp "${SOURCE_ARCHIVE}" "${DEST_ARCHIVE}"
write_metadata

echo "synced ${DEST_ARCHIVE}"
