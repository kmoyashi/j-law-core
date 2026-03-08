"""Python binding tests bootstrap."""

from __future__ import annotations

import os
import platform
import sys
from pathlib import Path


def _shared_library_filename() -> str:
    system = platform.system()
    if system == "Windows":
        return "j_law_c_ffi.dll"
    if system == "Darwin":
        return "libj_law_c_ffi.dylib"
    return "libj_law_c_ffi.so"


def pytest_sessionstart(session) -> None:
    """Build j-law-c-ffi once when tests run from the repo checkout."""
    del session

    repo_root = Path(__file__).resolve().parents[3]
    package_root = repo_root / "crates" / "j-law-python"
    sys.path.insert(0, str(package_root))
    from j_law_python import build_support

    target = build_support.rust_target_triple()
    manifest = build_support.repo_manifest_path(package_root)
    if target is not None:
        release_lib = build_support.built_shared_library_path(
            manifest,
            "release",
            target=target,
        )
        debug_lib = build_support.built_shared_library_path(
            manifest,
            "debug",
            target=target,
        )
    else:
        filename = _shared_library_filename()
        release_lib = repo_root / "target" / "release" / filename
        debug_lib = repo_root / "target" / "debug" / filename

    if release_lib.is_file() or debug_lib.is_file():
        existing = release_lib if release_lib.is_file() else debug_lib
        os.environ[build_support.ENV_LIBRARY_PATH] = str(existing)
        return

    built_library = build_support.build_shared_library(
        package_root=package_root,
        profile="debug",
    )
    os.environ[build_support.ENV_LIBRARY_PATH] = str(built_library)
