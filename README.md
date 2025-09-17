# ATTP-core
High performance core library for AgentHub V2, written in Rust with Python bindings.

## 🚀 Getting started
This project requires Rust and Python to be installed on your system.

Requirements:
- Rust (latest stable version recommended)
- Python 3.11
- Cargo (comes with Rust)
- Maturin (for building Python bindings)
- PyO3 (for Python bindings in Rust)
- PyO3-Async-Runtimes (for async support in PyO3)

## 🔧 Installation
To build and install the library, run:
```bash
maturin develop --release
```

Make sure that you have all the required dependencies installed, processor architecture matters. 
For more details, refer to the [Maturin documentation](https://maturin.rs/).

## 🧪 Building
For x86_64 architecture:
```bash
sudo docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release -i python3.11 -o dist
```

For ARM64 architecture:
```bash
sudo docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin:v1.9.3@sha256:cc6529e300f23dfa34243e7ad4f27571e06a7eae498aa133955c579ab5905718 build --release -i python3.11 -o dist
```
This will create a wheel file in the `dist` directory.

## 📦 Usage
After building, you can install the package using pip:
```bash
pip install dist/attp_core-0.1.2-cp311-cp311-linux_x86_64.whl
```
Replace the wheel file name with the one generated in your `dist` directory.

## 📝 License
This project is licensed under enterprise license - see the [LICENSE](LICENSE) file for details.