# Lighthouse

## Installation

You need to install both the `lighthouse` and `lcli` commands, so it's recommended to install them from source.

```sh
sudo apt update && sudo apt install -y git gcc g++ make cmake pkg-config llvm-dev libclang-dev clang
git clone https://github.com/sigp/lighthouse.git
cd lighthouse
git checkout v5.3.0 # The latest tested version
make
make install-lcli
```

Or consult the [official page](https://lighthouse-book.sigmaprime.io/installation-source.html) for the installation.
