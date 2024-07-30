#!/usr/bin/env bash

source ./scripts/util.sh
set -eu

mkdir -p $EXECUTION_DIR

# Initialize the reth nodes' directories
for (( node=0; node<$NODE_COUNT; node++ )); do
    el_data_dir $node

    $RETH_CMD init --datadir $el_data_dir --chain $ROOT/metadata/genesis.json 2>/dev/null
    echo "Initialized the data directory $el_data_dir"
done

# Set the IP address for the bootnode
yq -i ".hosts.bootnode.ip_addr = \"$BOOTNODE_IP\"" $SHADOW_CONFIG_FILE
# The "bootnode" process for the bootnode
args="-nodekey $(realpath ./assets/execution/boot.key) -verbosity 5 -addr :$EL_BOOTNODE_PORT -nat extip:$BOOTNODE_IP"
yq -i ".hosts.bootnode.processes += { \"path\": \"bootnode\", \"args\": \"$args\", \"expected_final_state\": \"running\" }" $SHADOW_CONFIG_FILE
log_shadow_config "the geth bootnode"

boot_enode="$(cat ./assets/execution/boot.enode)@$BOOTNODE_IP:0?discport=$EL_BOOTNODE_PORT"

# The reth process for each node
for (( node=0; node<$NODE_COUNT; node++ )); do
    el_data_dir $node
    node_ip $node

    args="node \
--chain $(realpath $ROOT/metadata/genesis.json) \
--datadir $(realpath $el_data_dir) \
--authrpc.port $EL_NODE_RPC_PORT \
--authrpc.jwtsecret $(realpath $ROOT/jwt/jwtsecret) \
--port $EL_NODE_PORT \
--db.log-level notice \
--bootnodes $boot_enode \
--nat extip:$ip \
--ipcdisable \
--log.file.filter trace \
--log.file.directory $(realpath $el_data_dir)
"
    yq -i ".hosts.node$node.processes += { \"path\": \"$RETH_CMD\", \
\"args\": \"$args\", \
\"environment\": { \"RUST_BACKTRACE\": \"full\" }, \
\"expected_final_state\": \"running\", \
\"start_time\": \"$(($node + 5))s\" \
 }" $SHADOW_CONFIG_FILE
    log_shadow_config "the reth process of the node #$node"
done
