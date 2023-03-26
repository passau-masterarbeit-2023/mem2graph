from setuptools import setup
from setuptools_rust import Binding, RustExtension

# compile rust lib: python setup.py build_ext --inplace

setup(
    name="mem_to_graph",
    version="0.1",
    rust_extensions=[
        RustExtension(
            "mem_to_graph.mem_to_graph",
            "Cargo.toml",
            binding=Binding.PyO3
        )
    ],
    packages=["mem_to_graph"],
    zip_safe=False,
)
