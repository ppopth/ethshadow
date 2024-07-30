#!/usr/bin/env bash

source ./scripts/util.sh
set -eu

mkdir -p $CONSENSUS_DIR
mkdir -p $BUILD_DIR

validators_present=0
if test -e $BUILD_DIR/validator_keys/mnemonic.txt; then
    if [[ $(< $BUILD_DIR/validator_keys/mnemonic.txt) != "$MNEMONIC" ]]; then
        rm -rf $BUILD_DIR/validator_keys
    else
        # Check how many validators we have already generated
        validators_present=$(find $BUILD_DIR/validator_keys/secrets -name "*" -print | wc -l)
    fi
fi

if test $validators_present -lt $VALIDATOR_COUNT; then
    docker run --rm -it -u $UID -v $BUILD_DIR:/data --entrypoint=eth2-val-tools \
        ethpandaops/ethereum-genesis-generator:3.3.5 \
        keystores --insecure --out-loc /data/validator_keys --source-mnemonic "$MNEMONIC" \
        --source-min $validators_present --source-max $VALIDATOR_COUNT 

    echo $MNEMONIC > $BUILD_DIR/validator_keys/mnemonic.txt
fi

validators_per_node=$(($VALIDATOR_COUNT/$NODE_COUNT))

for (( node=0; node<$NODE_COUNT; node++ )); do
    cl_data_dir $node
    mkdir -p $cl_data_dir/secrets
    mkdir -p $cl_data_dir/validators

    find $BUILD_DIR/validator_keys/secrets -mindepth 1 -maxdepth 1 | \
        head -$((($node + 1) * $validators_per_node)) | tail -$validators_per_node | xargs cp -t $cl_data_dir/secrets
    find $BUILD_DIR/validator_keys/keys -mindepth 1 -maxdepth 1 | \
        head -$((($node + 1) * $validators_per_node)) | tail -$validators_per_node | xargs cp -rt $cl_data_dir/validators
done

lcli \
    generate-bootnode-enr \
    --testnet-dir $ROOT/metadata \
    --ip $BOOTNODE_IP \
    --udp-port $CL_BOOTNODE_PORT \
    --tcp-port $CL_BOOTNODE_PORT \
    --genesis-fork-version $GENESIS_FORK_VERSION \
    --output-dir $CL_BOOTNODE_DIR

bootnode_enr=$(cat $CL_BOOTNODE_DIR/enr.dat)
echo "- $bootnode_enr" > $CONSENSUS_DIR/boot_enr.yaml
echo "Generated $CONSENSUS_DIR/boot_enr.yaml"

# The "lighthouse boot_node" process for the bootnode
# --disable-packet-filter is necessary because it's involed in rate limiting and nodes per IP limit
# See https://github.com/sigp/discv5/blob/v0.1.0/src/socket/filter/mod.rs#L149-L186
args="\
--testnet-dir $(realpath $ROOT/metadata) \
boot_node \
--port $CL_BOOTNODE_PORT \
--disable-packet-filter \
--network-dir $(realpath $CL_BOOTNODE_DIR)
"
yq -i ".hosts.bootnode.processes += { \
    \"path\": \"$LIGHTHOUSE_CMD\", \
    \"args\": \"$args\", \
    \"expected_final_state\": \"running\" \
}" $SHADOW_CONFIG_FILE
log_shadow_config "the lighthouse bootnode"

# The lighthouse beacon node process for each node
for (( node=0; node<$NODE_COUNT; node++ )); do
    cl_data_dir $node
    node_ip $node
    # --disable-packet-filter is necessary because it's involed in rate limiting and nodes per IP limit
    # See https://github.com/sigp/discv5/blob/v0.1.0/src/socket/filter/mod.rs#L149-L186
    args="\
--testnet-dir $(realpath $ROOT/metadata) \
beacon_node \
--datadir $(realpath $cl_data_dir) \
--execution-endpoint http://localhost:$EL_NODE_RPC_PORT \
--execution-jwt $(realpath $ROOT/jwt/jwtsecret) \
--boot-nodes "$bootnode_enr" \
--enr-address $ip \
--enr-udp-port $CL_BEACON_NODE_PORT \
--enr-tcp-port $CL_BEACON_NODE_PORT \
--port $CL_BEACON_NODE_PORT \
--http \
--http-port $CL_BEACON_NODE_HTTP_PORT \
--disable-quic \
--disable-upnp \
--subscribe-all-subnets \
--disable-packet-filter
"
    yq -i ".hosts.node$node.processes += { \
        \"path\": \"$LIGHTHOUSE_CMD\", \
        \"args\": \"$args\", \
        \"environment\": { \"RUST_LOG\": \"gossipsub=trace\" }, \
        \"expected_final_state\": \"running\" \
    }" $SHADOW_CONFIG_FILE
    log_shadow_config "the lighthouse beacon node process of the node #$node"
done

# The lighthouse validator client process for each node
for (( node=0; node<$NODE_COUNT; node++ )); do
    cl_data_dir $node

    args="\
--testnet-dir $(realpath $ROOT/metadata) \
validator_client \
--datadir $(realpath $cl_data_dir) \
--beacon-nodes http://localhost:$CL_BEACON_NODE_HTTP_PORT \
--suggested-fee-recipient 0xf97e180c050e5Ab072211Ad2C213Eb5AEE4DF134 \
--init-slashing-protection
"
    yq -i ".hosts.node$node.processes += { \
        \"path\": \"$LIGHTHOUSE_CMD\", \
        \"args\": \"$args\", \
        \"expected_final_state\": \"running\" \
    }" $SHADOW_CONFIG_FILE
    log_shadow_config "the lighthouse validator client process of the node #$node"
done
