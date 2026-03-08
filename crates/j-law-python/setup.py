from __future__ import annotations

from pathlib import Path
import sys

from setuptools import Distribution
from setuptools import setup
from setuptools.command.build_py import build_py as _build_py
from setuptools.command.sdist import sdist as _sdist

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))

from j_law_python import build_support

try:
    from wheel.bdist_wheel import bdist_wheel as _bdist_wheel
except ImportError:  # pragma: no cover
    _bdist_wheel = None


class BinaryDistribution(Distribution):
    def has_ext_modules(self) -> bool:
        return True


class BuildPy(_build_py):
    def run(self) -> None:
        super().run()
        self._built_library_output = str(
            build_support.copy_shared_library(
                Path(self.build_lib),
                package_root=ROOT,
                profile="release",
            )
        )

    def get_outputs(self, include_bytecode: bool = True):  # type: ignore[override]
        outputs = super().get_outputs(include_bytecode=include_bytecode)
        built_library = getattr(self, "_built_library_output", None)
        if built_library is not None:
            outputs.append(built_library)
        return outputs


class Sdist(_sdist):
    def run(self) -> None:
        build_support.prepare_vendored_rust(ROOT)
        super().run()


cmdclass = {
    "build_py": BuildPy,
    "sdist": Sdist,
}

if _bdist_wheel is not None:
    class BdistWheel(_bdist_wheel):
        def finalize_options(self) -> None:
            super().finalize_options()
            self.root_is_pure = False

    cmdclass["bdist_wheel"] = BdistWheel


setup(
    distclass=BinaryDistribution,
    cmdclass=cmdclass,
    include_package_data=True,
    zip_safe=False,
)
