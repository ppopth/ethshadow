#!/usr/bin/env bash

source ./scripts/util.sh
set -eu

mkdir -p $EXECUTION_DIR

new_account() {
    local node=$1
    local datadir=$2

    # Generate a new account for each geth node
    address=$(geth --datadir $datadir account new --password $ROOT/password 2>/dev/null | grep -o "0x[0-9a-fA-F]*")
    echo "Generated an account with address $address for geth node $node and saved it at $datadir"
    echo $address > $datadir/address

    # Add the account into the genesis state
    alloc=$(echo $genesis | jq ".alloc + { \"${address:2}\": { \"balance\": \"$INITIAL_BALANCE\" } }")
    genesis=$(echo $genesis | jq ". + { \"alloc\": $alloc }")
}

genesis=$(cat $GENESIS_TEMPLATE_FILE)
for (( node=1; node<=$NODE_COUNT; node++ )); do
    el_data_dir $node
    new_account "#$node" $el_data_dir
done

new_account "'signer'" $SIGNER_EL_DATADIR

# Add the extradata
zeroes() {
    for i in $(seq $1); do
        echo -n "0"
    done
}
address=$(cat $SIGNER_EL_DATADIR/address)
extra_data="0x$(zeroes 64)${address:2}$(zeroes 130)"
genesis=$(echo $genesis | jq ". + { \"extradata\": \"$extra_data\" }")

# Add the terminal total difficulty
config=$(echo $genesis | jq ".config + { \"chainId\": "$NETWORK_ID", \"terminalTotalDifficulty\": "$TERMINAL_TOTAL_DIFFICULTY", \"clique\": { \"period\": "$SECONDS_PER_ETH1_BLOCK", \"epoch\": 30000 } }")
genesis=$(echo $genesis | jq ". + { \"config\": $config }")

# Generate the genesis state
echo $genesis > $GENESIS_FILE
echo "Generated $GENESIS_FILE"

# Initialize the geth nodes' directories
for (( node=1; node<=$NODE_COUNT; node++ )); do
    el_data_dir $node
    datadir=$el_data_dir

    geth init --datadir $datadir $GENESIS_FILE 2>/dev/null
    echo "Initialized the data directory $datadir with $GENESIS_FILE"
done

geth init --datadir $SIGNER_EL_DATADIR $GENESIS_FILE 2>/dev/null
echo "Initialized the data directory $SIGNER_EL_DATADIR with $GENESIS_FILE"

# Set the IP address for the bootnode
yq -i ".hosts.bootnode.ip_addr = \"$EL_BOOTNODE_IP\"" $SHADOW_CONFIG_FILE
# Set the arguments for the bootnode's geth command
yq -i ".hosts.bootnode.processes[].args = \"-nodekey $(realpath ./assets/execution/boot.key) -verbosity 5 -addr :$EL_BOOTNODE_PORT\"" $SHADOW_CONFIG_FILE

boot_enode="$(cat ./assets/execution/boot.enode)@$EL_BOOTNODE_IP:0?discport=$EL_BOOTNODE_PORT"

# Set the arguments for the signing node's geth command
address=$(cat $SIGNER_EL_DATADIR/address)
args="\
--datadir $(realpath $SIGNER_EL_DATADIR) \
--port $SIGNER_PORT \
--http \
--http.port $SIGNER_HTTP_PORT \
--allow-insecure-unlock \
--bootnodes $boot_enode \
--networkid $NETWORK_ID \
--nat extip:$SIGNER_IP \
--ipcdisable \
--unlock $address \
--password $(realpath $ROOT/password) \
--mine \
"
yq -i ".hosts.signernode.processes[].args = \"$args\"" $SHADOW_CONFIG_FILE
yq -i ".hosts.signernode.ip_addr = \"$SIGNER_IP\"" $SHADOW_CONFIG_FILE

# Set the arguments for each node's geth command
for (( node=1; node<=$NODE_COUNT; node++ )); do
    el_data_dir $node
    address=$(cat $el_data_dir/address)
    node_ip $node

    args="\
--datadir $(realpath $el_data_dir) \
--port $EL_NODE_PORT \
--bootnodes $boot_enode \
--networkid $NETWORK_ID \
--nat extip:$ip \
--ipcdisable \
--unlock $address \
--password $(realpath $ROOT/password) \
"
    yq -i ".hosts.node$node.processes += { \"path\": \"geth\", \"args\": \"$args\" }" $SHADOW_CONFIG_FILE
done
