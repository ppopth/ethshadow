# Discrete-event Ethereum network simulation

Simulate a full Ethereum network using [Shadow](https://shadow.github.io/), a discrete-event network simulator. The way we run
the Ethereum network is similar to and adopted from [Local Ethereum Testnet](https://github.com/ppopth/local-testnet).
That is, we use [lighthouse](https://github.com/sigp/lighthouse) and [geth](https://github.com/ethereum/go-ethereum) as
the consensus client and execution client respectively. Please read the mentioned link for more detail.

## Install Dependencies

You can follow the follwing instructions to install the dependencies. Please note that we use our own version of Shadow.
```bash
# Install geth and bootnode
sudo add-apt-repository -y ppa:ethereum/ethereum
sudo apt-get update
sudo apt-get install -y ethereum

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install lighthouse and lcli
sudo apt-get install -y git gcc g++ make cmake pkg-config llvm-dev libclang-dev clang protobuf-compiler
git clone https://github.com/sigp/lighthouse.git
cd lighthouse
git checkout stable
make
make install-lcli
cd ..

# Install Node.js
sudo apt-get install -y nodejs npm

# Install jq
sudo apt-get install -y jq

# Install yq
snap install yq

# Install our version of Shadow
sudo apt-get install -y glib2.0-dev
git clone https://github.com/ppopth/shadow.git
cd shadow
./setup build
./setup install
# If ~/.local/bin is not already in your PATH, run the following command
echo 'export PATH="${PATH}:${HOME}/.local/bin"' >> ~/.bashrc && source ~/.bashrc
cd ..
```

## Run the simulation
```bash
git clone https://github.com/ppopth/ethereum-shadow.git
cd ethereum-shadow
./run.sh
```
By default, the number of nodes will be 4 and the number of validators will be 80. You can change them by setting the environment variables.
```bash
NODE_COUNT=2 VALIDATOR_COUNT=10 ./run.sh
```
*Note: new versions of geth and lighthouse can cause the simulation to run unsuccessfully because they will probably contain some syscalls that
don't support in Shadow yet. As of this writing, it works with geth 1.10.26, lighthouse 3.4.0, and rust 1.65.0*

## Simulation Result

After running the simulation, the following files and directories are probably the ones you want to look at.
* `./data/shadow.yaml` - The generated Shadow config file which we use to run the simulation. Note that the file is generated and not included in
the git repository.
* `./data/shadow.log` - The stdout of the Shadow process is redirected to this file.
* `./data/shadow/hosts/` - The directory which contains the stdout and stderr of every process (including geth and lighthouse) of every node.

Note that the timestamps shown in `./data/shadow/hosts` are not from the clock of the physical machine, but they are from the clock in the simulation itself
which are similar to the ones you will get from the real Ethereum network. This is the main advantage we give over the one in [Local Ethereum Testnet](https://github.com/ppopth/local-testnet)
