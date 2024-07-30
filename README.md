# Discrete-event Ethereum network simulation

*This is a fork of [Pop's code](https://github.com/ppopth/ethereum-shadow), and part of my [EPF Project](https://github.com/eth-protocol-fellows/cohort-five/blob/main/projects/network-simulations-with-shadow.md).*

**This is still very unstable, and the documentation below is not updated yet! You need to use the `shadow` branch my forks of [Geth (for `bootnode`)](https://github.com/dknopik/go-ethereum/tree/shadow) and [Reth](https://github.com/dknopik/reth/tree/shadow) in order to be able to run the simulation. For Lighthouse, you need a recent `unstable` build.**

Simulate a full Ethereum network using [Shadow](https://shadow.github.io/), a discrete-event network simulator. The way we run
the Ethereum network is similar to and adopted from [Local Ethereum Testnet](https://github.com/ppopth/local-testnet).
That is, we use [Lighthouse](https://github.com/sigp/lighthouse) and [Reth](https://github.com/ethereum/go-ethereum) as
the consensus client and execution client respectively. Please read the mentioned link for more detail.

The advantages of using ethereum-shadow is that
1. It already includes everything in the simulation (e.g. libp2p, discv5, etc).
2. It uses the same software as the mainnet and the public testnets.
3. If there is any upgades in lighthouse, we can integrate those upgrades easily in the simulation.

If you want to simulate a new Ethereum protocol, what you need to do is just to implement it in lighthouse or geth and
run it using this simulator.

## Install Dependencies

You can follow the follwing instructions to install the dependencies. Please note that we use our own version of Shadow.
```bash
# Install Go
curl -OL https://go.dev/dl/go1.20.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.20.linux-amd64.tar.gz
# If /usr/local/go/bin is not already in your PATH, run the following command
echo 'export PATH="${PATH}:/usr/local/go/bin"' >> ~/.bashrc && source ~/.bashrc

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install lighthouse and lcli
sudo apt-get install -y git gcc g++ make cmake pkg-config llvm-dev libclang-dev clang protobuf-compiler
git clone https://github.com/sigp/lighthouse.git
cd lighthouse
git checkout v3.4.0
make
make install-lcli
cd ..

# Install geth and bootnode
git clone https://github.com/ethereum/go-ethereum.git
cd go-ethereum
git checkout v1.10.26
make all
cp ./build/bin/geth ./build/bin/bootnode /usr/bin
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
NODE_COUNT=8 VALIDATOR_COUNT=40 ./run.sh
```
*Note: new versions of geth and lighthouse can cause the simulation to run unsuccessfully because they will probably contain some syscalls that
don't support in Shadow yet. As of this writing, it works with geth 1.10.26, lighthouse 3.4.0, and rust 1.65.0*

If you want to specify the paths for your own geth and lighthouse binaries, you can do so by setting RETH_CMD and LIGHTHOUSE_CMD environment variables.
```bash
RETH_CMD=~/repos/go-ethereum/build/bin/geth LIGHTHOUSE_CMD=~/repos/lighthouse/target/release/lighthouse ./run.sh
```

## Simulation Result

After running the simulation, the following files and directories are probably the ones you want to look at.
* `./data/shadow.yaml` - The generated Shadow config file which we use to run the simulation. Note that the file is generated and not included in
the git repository.
* `./data/shadow.log` - The stdout of the Shadow process is redirected to this file.
* `./data/shadow/hosts/` - The directory which contains the stdout and stderr of every process (including geth and lighthouse) of every node.

Note that the timestamps shown in `./data/shadow/hosts` are not from the clock of the physical machine, but they are from the clock in the simulation itself
which are similar to the ones you will get from the real Ethereum network. This is the main advantage we give over the one in [Local Ethereum Testnet](https://github.com/ppopth/local-testnet)

## Network Topology

Currently we set each node to have 20Mbps for downloading and another 20Mbps for uploading. Each pair of nodes has the latency of 100ms (which is the average latency of the nodes in discv5 network, see https://notes.ethereum.org/@pop/discv5-network-measurement). You can see and change the default setting at [./assets/shadow.template.yaml](https://github.com/ppopth/ethereum-shadow/blob/main/assets/shadow.template.yaml).

In the future, we have a plan to assign each node the physical location so that the latencies among nodes will be more realistic.

## Scale to hundreds of nodes

### Kernel Configuration
If you want to run the simulation with a lot of nodes, you need to change some limits in the kernel configuration because we will use more resources
than the default configuration allows. However, we will only quickly write a bunch of commands here for those who don't want to get into detail. If you want to know why we need to change each of these configurations, please read https://shadow.github.io/docs/guide/system_configuration.html
```bash
echo "fs.nr_open = 10485760" | sudo tee -a /etc/sysctl.conf
echo "fs.file-max = 10485760" | sudo tee -a /etc/sysctl.conf
sudo systemctl set-property user-$UID.slice TasksMax=infinity
echo "vm.max_map_count = 1073741824" | sudo tee -a /etc/sysctl.conf
echo "kernel.pid_max = 4194304" | sudo tee -a /etc/sysctl.conf
echo "kernel.threads-max = 4194304" | sudo tee -a /etc/sysctl.conf
```
Add the followings lines to `/etc/security/limits.conf`, but change `myname` to your username in the machine that you will use to run the simulation.
```
myname soft nofile 10485760
myname hard nofile 10485760
myname soft nproc unlimited
myname hard nproc unlimited
myname soft stack unlimited
myname hard stack unlimited
```
If you use the GUI login in your machine, you also need to add the following line in both `/etc/systemd/user.conf` and `/etc/systemd/system.conf`.
```
DefaultLimitNOFILE=10485760
```
Reboot your machine to make the change effective.
```
reboot
```

### Swap Space

If you run a lot of nodes, your memory space is probably not enough. You probably need to create some swap space in your storage device. In this example, we will create a 16GB swap file at `/swapfile`.
```bash
sudo fallocate -l 16G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
# Backup the old file
sudo cp /etc/fstab /etc/fstab.bak
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```
Run `free -h` to check that the swap space is already created.

### Parallelism

You probably want to run the simulation in a multi-core machine. By default, `./run.sh` uses only 1 core to run the simulation, so you need to provide
the degree of parallelism with which you want to it to run. You can put the degree in the environment variable called `PARALLELISM`.

It's best to put the parallelism to be the same as the number of cores you have. You can get your number of cores by running `nproc`.
```bash
PARALLELISM=$(nproc) ./run.sh
```

### Issue with Spinlocks

Since lighthouse uses the crated called [spin](https://github.com/mvdnes/spin-rs), you will probably encounter that some nodes consume a lot of CPU without making any simulation progress. The solution to this problem is to override the spin crate with our own crate. Run the following commands to do so.
```bash
git clone https://github.com/ppopth/spin-rs.git
cd lighthouse
echo -e '\n[patch.crates-io.spin]\npath = "../spin-rs"' | tee -a Cargo.toml
make
```

### Simulations

* Jan 22, 2023: I tried running a simulation with 20 nodes and 400 validators on a Digital Ocean instance with 8 vCPUs, 16GB of memory. It took me 45 minutes to run the simulation of 30 minutes of the network.
* Jan 22, 2023: I tried running a simulation with 100 nodes and 2,000 validators on a Digital Ocean instance with 8 vCPUs, 16GB of memory, and 16GB of swap space in SSD. It took me 3 hours to run the simulation of 30 minutes of the network.
* Jan 25, 2023: I tried running a simulation with 200 nodes and 4,000 validators on a Digital Ocean instance with 8 vCPUs, 16GB of memory, and 64GB of swap space in SSD. It took me 17 hours to run the simulation of 30 minutes of the network.
* Feb 26, 2023: I tried running a simulation with 500 nodes and 2,000 validators on a Digital Ocean instance with 32 vCPUs, 256GB of memory with no swap space. It took me an hour to run the simulation of 10 minutes of the network.
* Feb 26, 2023: I tried running a simulation with 1,000 nodes and 2,000 validators on a Digital Ocean instance with 32 vCPUs, 256GB of memory, and 64GB of swap space in SSD. It took me 2 hours to run the simulation of 10 minutes of the network.
