"""Build and runtime helpers for the Python C FFI binding."""

from __future__ import annotations

import os
import platform
import shutil
import subprocess
from pathlib import Path

FFI_VERSION = 4
ENV_LIBRARY_PATH = "JLAW_PYTHON_C_FFI_LIB"
PACKAGE_ROOT = Path(__file__).resolve().parents[1]

WORKSPACE_TOML = """[workspace]
members = [
    "crates/j-law-core",
    "crates/j-law-registry",
    "crates/j-law-c-ffi",
]
resolver = "2"

[workspace.lints.clippy]
disallowed_methods = "warn"
disallowed_types = "warn"
disallowed_macros = "warn"
"""

COPY_MAP = {
    "crates/j-law-c-ffi": ("Cargo.toml", "src", "j_law_c_ffi.h"),
    "crates/j-law-core": ("Cargo.toml", "src"),
    "crates/j-law-registry": ("Cargo.toml", "src", "data"),
}


def shared_library_filename() -> str:
    system = platform.system()
    if system == "Windows":
        return "j_law_c_ffi.dll"
    if system == "Darwin":
        return "libj_law_c_ffi.dylib"
    return "libj_law_c_ffi.so"


def packaged_shared_library_path(package_root: Path = PACKAGE_ROOT) -> Path:
    return package_root / "j_law_python" / "native" / shared_library_filename()


def _target_machine() -> str:
    """Return the target machine architecture, respecting cross-compilation env vars.

    cibuildwheel sets ARCHFLAGS (e.g. ``-arch x86_64``) on macOS when
    cross-compiling, which takes precedence over the host machine architecture.
    """
    archflags = os.environ.get("ARCHFLAGS", "")
    if "-arch x86_64" in archflags:
        return "x86_64"
    if "-arch arm64" in archflags:
        return "arm64"
    return platform.machine().lower()


def _is_musl_linux() -> bool:
    """Return True when the current Linux environment uses musl libc.

    cibuildwheel sets AUDITWHEEL_PLAT to a value like
    ``musllinux_1_2_x86_64`` inside musllinux containers, which is the
    most reliable indicator.  As a fallback we look for the musl dynamic
    linker that is present on Alpine and similar distributions.
    """
    if "musl" in os.environ.get("AUDITWHEEL_PLAT", ""):
        return True
    import glob
    return bool(glob.glob("/lib/ld-musl-*.so*"))


def rust_target_triple() -> str | None:
    system = platform.system()
    machine = _target_machine()

    if system == "Darwin":
        if machine in {"arm64", "aarch64"}:
            return "aarch64-apple-darwin"
        if machine in {"x86_64", "amd64"}:
            return "x86_64-apple-darwin"
    if system == "Linux":
        musl = _is_musl_linux()
        if machine in {"x86_64", "amd64"}:
            return "x86_64-unknown-linux-musl" if musl else "x86_64-unknown-linux-gnu"
        if machine in {"arm64", "aarch64"}:
            return "aarch64-unknown-linux-musl" if musl else "aarch64-unknown-linux-gnu"
    if system == "Windows":
        if machine in {"x86_64", "amd64"}:
            return "x86_64-pc-windows-msvc"
        if machine in {"arm64", "aarch64"}:
            return "aarch64-pc-windows-msvc"

    return None


def vendored_workspace_root(package_root: Path = PACKAGE_ROOT) -> Path:
    return package_root / "vendor" / "rust"


def vendored_manifest_path(package_root: Path = PACKAGE_ROOT) -> Path:
    return vendored_workspace_root(package_root) / "Cargo.toml"


def repo_workspace_root(package_root: Path = PACKAGE_ROOT) -> Path:
    return package_root.parents[1]


def repo_manifest_path(package_root: Path = PACKAGE_ROOT) -> Path:
    return repo_workspace_root(package_root) / "Cargo.toml"


def manifest_path(package_root: Path = PACKAGE_ROOT) -> Path | None:
    vendored_manifest = vendored_manifest_path(package_root)
    if vendored_manifest.is_file():
        return vendored_manifest

    repo_manifest = repo_manifest_path(package_root)
    if repo_manifest.is_file():
        return repo_manifest

    return None


def built_shared_library_path(
    manifest: Path,
    profile: str = "release",
    target: str | None = None,
) -> Path:
    target_dir = manifest.parent / "target"
    if target is not None:
        target_dir = target_dir / target
    return target_dir / profile / shared_library_filename()


def shared_library_candidates(package_root: Path = PACKAGE_ROOT) -> list[Path]:
    candidates: list[Path] = []
    env_path = os.environ.get(ENV_LIBRARY_PATH)
    if env_path:
        candidates.append(Path(env_path))

    candidates.append(packaged_shared_library_path(package_root))

    repo_manifest = repo_manifest_path(package_root)
    if repo_manifest.is_file():
        target = rust_target_triple()
        if target is not None:
            candidates.append(
                built_shared_library_path(repo_manifest, "release", target=target)
            )
            candidates.append(
                built_shared_library_path(repo_manifest, "debug", target=target)
            )
        candidates.append(built_shared_library_path(repo_manifest, "release"))
        candidates.append(built_shared_library_path(repo_manifest, "debug"))
    deduped: list[Path] = []
    for candidate in candidates:
        if candidate not in deduped:
            deduped.append(candidate)
    return deduped


def resolve_shared_library_path(package_root: Path = PACKAGE_ROOT) -> Path | None:
    for candidate in shared_library_candidates(package_root):
        if candidate.is_file():
            return candidate
    return None


def prepare_vendored_rust(package_root: Path = PACKAGE_ROOT) -> Path:
    vendor_root = vendored_workspace_root(package_root)
    repo_root = repo_workspace_root(package_root)

    shutil.rmtree(vendor_root, ignore_errors=True)
    vendor_root.mkdir(parents=True, exist_ok=True)
    (vendor_root / "Cargo.toml").write_text(WORKSPACE_TOML, encoding="utf-8")

    cargo_lock = repo_root / "Cargo.lock"
    if cargo_lock.is_file():
        shutil.copy2(cargo_lock, vendor_root / "Cargo.lock")

    for crate_dir, entries in COPY_MAP.items():
        for entry in entries:
            source = repo_root / crate_dir / entry
            destination = vendor_root / crate_dir / entry
            destination.parent.mkdir(parents=True, exist_ok=True)
            if source.is_dir():
                shutil.copytree(source, destination, dirs_exist_ok=True)
            else:
                shutil.copy2(source, destination)

    return vendor_root


def build_shared_library(
    package_root: Path = PACKAGE_ROOT,
    profile: str = "release",
) -> Path:
    manifest = manifest_path(package_root)
    if manifest is None:
        raise RuntimeError("Cargo workspace for j-law-c-ffi was not found.")

    command = [
        "cargo",
        "build",
        "-p",
        "j-law-c-ffi",
        "--manifest-path",
        str(manifest),
    ]
    target = rust_target_triple()
    if target is not None:
        command.extend(["--target", target])
    if profile == "release":
        command.append("--release")

    env = None
    if target is not None and target.endswith("-linux-musl"):
        # musl ターゲットはデフォルトで crt_static_allows_dylibs = false のため
        # cdylib クレートタイプが無効化される。
        # -C target-feature=-crt-static で動的リンクを有効にして cdylib をビルド可能にする。
        env = os.environ.copy()
        existing = env.get("RUSTFLAGS", "")
        env["RUSTFLAGS"] = (existing + " -C target-feature=-crt-static").strip()

    subprocess.run(command, check=True, cwd=manifest.parent, env=env)

    built_library = built_shared_library_path(manifest, profile, target=target)
    if not built_library.is_file():
        raise RuntimeError(f"built shared library was not found: {built_library}")

    return built_library


def copy_shared_library(
    destination_root: Path,
    package_root: Path = PACKAGE_ROOT,
    profile: str = "release",
) -> Path:
    built_library = build_shared_library(package_root=package_root, profile=profile)
    target = destination_root / "j_law_python" / "native" / shared_library_filename()
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(built_library, target)
    return target
