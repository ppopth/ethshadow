FROM ubuntu

RUN apt-get update

RUN apt-get install -y git curl build-essential

RUN apt-get install -y \
    cmake \
    findutils \
    libclang-dev \
    libc-dbg \
    libglib2.0-0 \
    libglib2.0-dev \
    make \
    netbase \
    python3 \
    python3-networkx \
    xz-utils \
    util-linux \
    gcc \
    g++

ENV PATH="/root/.cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN git clone https://github.com/shadow/shadow.git && \
    cd shadow && \
    ./setup build --clean && \
    ./setup install && \
    cd ..

RUN apt install -y git gcc g++ make cmake pkg-config llvm-dev libclang-dev clang

RUN git clone https://github.com/sigp/lighthouse.git && \
    cd lighthouse && \
    git checkout v5.3.0 && \
    make && \
    make install-lcli && \
    lighthouse -V && \
    cd ..

ENV PATH="$PATH:/usr/local/go/bin"

RUN curl -LO https://go.dev/dl/go1.23.7.linux-amd64.tar.gz && \
    rm -rf /usr/local/go && tar -C /usr/local -xzf go1.23.7.linux-amd64.tar.gz

RUN git clone https://github.com/ethereum/go-ethereum.git && \
    cd go-ethereum && \
    git checkout v1.14.11 && \
    make all && \
    cp build/bin/geth /usr/local/bin/geth && \
    cp build/bin/bootnode /usr/local/bin/bootnode && \
    geth -v && \
    cd ..

RUN curl -LO https://github.com/prysmaticlabs/prysm/releases/download/v5.3.0/beacon-chain-v5.3.0-linux-amd64 && \
    chmod +x beacon-chain-v5.3.0-linux-amd64

ADD . /ethshadow

RUN cd ethshadow && cargo install --path . && cd ..

RUN echo 'export PATH="${PATH}:/root/.local/bin"' >> ~/.bashrc
