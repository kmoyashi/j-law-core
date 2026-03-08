"""C ABI adapter tests for the Python binding."""

from __future__ import annotations

import datetime
from pathlib import Path

import pytest

from j_law_python import build_support
from j_law_python import consumption_tax
from j_law_python import real_estate
from j_law_python import _cgo


def test_abi_version_matches():
    assert _cgo.abi_version() == _cgo.ABI_VERSION


def test_env_library_path_has_highest_priority(tmp_path, monkeypatch):
    package_root = tmp_path / "workspace" / "crates" / "j-law-python"
    manifest = build_support.repo_manifest_path(package_root)
    env_lib = tmp_path / "override" / build_support.shared_library_filename()
    packaged_lib = (
        package_root
        / "j_law_python"
        / "native"
        / build_support.shared_library_filename()
    )
    release_lib = build_support.built_shared_library_path(manifest, "release")
    debug_lib = build_support.built_shared_library_path(manifest, "debug")

    manifest.parent.mkdir(parents=True, exist_ok=True)
    manifest.write_text("[workspace]\n", encoding="utf-8")

    for path in (env_lib, packaged_lib, release_lib, debug_lib):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"")

    monkeypatch.setenv("JLAW_PYTHON_CGO_LIB", str(env_lib))

    candidates = build_support.shared_library_candidates(package_root)

    assert candidates[0] == env_lib
    assert candidates[1] == packaged_lib
    assert release_lib in candidates
    assert debug_lib in candidates
    assert build_support.resolve_shared_library_path(package_root) == env_lib


def test_debug_library_is_last_fallback(tmp_path, monkeypatch):
    package_root = tmp_path / "workspace" / "crates" / "j-law-python"
    manifest = build_support.repo_manifest_path(package_root)
    manifest.parent.mkdir(parents=True, exist_ok=True)
    manifest.write_text("[workspace]\n", encoding="utf-8")
    target = build_support.rust_target_triple()
    if target is not None:
        debug_lib = build_support.built_shared_library_path(
            manifest,
            "debug",
            target=target,
        )
    else:
        debug_lib = build_support.built_shared_library_path(manifest, "debug")
    debug_lib.parent.mkdir(parents=True, exist_ok=True)
    debug_lib.write_bytes(b"")

    monkeypatch.delenv("JLAW_PYTHON_CGO_LIB", raising=False)

    assert build_support.resolve_shared_library_path(package_root) == debug_lib


def test_fixed_length_strings_are_restored():
    record = _cgo.calc_brokerage_fee(
        5_000_000,
        2024,
        8,
        1,
        False,
        False,
    )

    assert [step.label for step in record.breakdown] == ["tier1", "tier2", "tier3"]


def test_bool_flags_are_restored_as_bool():
    brokerage = _cgo.calc_brokerage_fee(
        8_000_000,
        2024,
        8,
        1,
        True,
        False,
    )
    tax = _cgo.calc_consumption_tax(
        100_000,
        2024,
        1,
        1,
        True,
    )

    assert isinstance(brokerage.low_cost_special_applied, bool)
    assert brokerage.low_cost_special_applied is True
    assert isinstance(tax.is_reduced_rate, bool)
    assert tax.is_reduced_rate is True


def test_error_buffer_is_raised_as_value_error():
    with pytest.raises(ValueError):
        consumption_tax.calc_consumption_tax(
            100_000,
            datetime.date(2016, 1, 1),
            is_reduced_rate=True,
        )


def test_loaded_library_path_exists():
    assert Path(_cgo.library_path()).is_file()


def test_public_api_still_uses_ctypes_backed_result():
    result = real_estate.calc_brokerage_fee(
        5_000_000,
        datetime.date(2024, 8, 1),
    )

    assert result.total_with_tax == 231_000
