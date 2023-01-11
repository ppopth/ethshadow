#!/usr/bin/env bash

source ./scripts/util.sh
set -eu

mkdir -p $CONSENSUS_DIR
mkdir -p $BUILD_DIR

if ! test -e $BUILD_DIR/deposit; then
    echo "$BUILD_DIR/deposit not found. Downloading it from https://github.com/ethereum/staking-deposit-cli"

    curl -s -L -o $ROOT/deposit.tar.gz "https://github.com/ethereum/staking-deposit-cli/releases/download/v2.3.0/staking_deposit-cli-76ed782-linux-amd64.tar.gz"
    tar xf $ROOT/deposit.tar.gz -C $ROOT

    mv $ROOT/staking_deposit-cli-76ed782-linux-amd64/deposit $BUILD_DIR/deposit
    rm -rf $ROOT/staking_deposit-cli-76ed782-linux-amd64 $ROOT/deposit.tar.gz

    echo "$BUILD_DIR/deposit downloaded"
fi

validator_count=0
if test -e $BUILD_DIR/validator_keys; then
    # Check how many validators we have already generated
    validator_count=$(find $BUILD_DIR/validator_keys -name "keystore*" -print | wc -l)
fi

if test $validator_count -lt $VALIDATOR_COUNT; then
    echo "Generating the credentials for all of $VALIDATOR_COUNT validators at $BUILD_DIR/validator_keys"

    # Generate only for the remaining validators
    # We use kiln because we have the same GENESIS_FORK_VERSION which is 0x70000069
    $BUILD_DIR/deposit \
        --language english \
        --non_interactive \
        existing-mnemonic \
        --num_validators $(expr $VALIDATOR_COUNT - $validator_count)\
        --mnemonic="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
        --validator_start_index $validator_count \
        --chain kiln \
        --keystore_password $(cat $ROOT/password) \
        --folder $BUILD_DIR

    echo "Done generating the credentials"
fi

if ! test -e ./web3/node_modules; then
    echo "The package ./web3 doesn't have node modules installed yet. Installing the node modules now"
    npm --prefix ./web3 install >/dev/null 2>/dev/null
    echo "Node modules are already installed"
fi

# Use the signing node as a node to deploy the deposit contract
env="NODE_PATH=$(realpath ./web3/node_modules)"
args="$(realpath ./web3/src/deploy-deposit-contract.js) \
--endpoint http://localhost:$SIGNER_HTTP_PORT \
--file $(realpath ./assets/deposit-contract.json) \
--address-out $(realpath $ROOT/deposit-address) \
--block-out $(realpath $CONSENSUS_DIR/deploy_block.txt) \
"

yq -i ".hosts.signernode.processes += { \
    \"path\": \"node\", \
    \"environment\": \"$env\", \
    \"args\": \"$args\", \
    \"start_time\": 10 \
}" $SHADOW_CONFIG_FILE
