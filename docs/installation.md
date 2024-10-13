# Installation

Only Linux is supported. For more details, see the
[Shadow documentation](https://shadow.github.io/docs/guide/supported_platforms.html).

## Install Go

See the [official page](https://go.dev/doc/install)

## Install Rust

See the [official page](https://www.rust-lang.org/tools/install)

## Install Docker

See the [install page](https://docs.docker.com/engine/install) and the [non-root user page](https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user) for the installation.

The Docker daemon must be running while Ethshadow prepares the simulation.

## Install Shadow and its dependencies

```sh
sudo apt-get install -y cmake findutils libclang-dev libc-dbg libglib2.0-0 libglib2.0-dev make netbase python3 python3-networkx xz-utils util-linux gcc g++
git clone https://github.com/shadow/shadow.git
cd shadow
./setup build --clean
./setup install
echo 'export PATH="${PATH}:/home/${USER}/.local/bin"' >> ~/.bashrc && source ~/.bashrc
```

Or consult the [official page](https://shadow.github.io/docs/guide/install_dependencies.html) for the installation.

## Install CL and EL clients

Ensure that all clients you want to use in the simulation are installed, see the [supported client page](supported-clients.md) for notes.

## Install Ethshadow

Install Ethshadow by running `cargo install --path .`
