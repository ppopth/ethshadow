#!/usr/bin/env bash

if ! test $(uname -s) = "Linux"; then
    echo "Only Linux is supported"
fi

check_cmd() {
    if ! command -v $1 >/dev/null; then
        echo -e "\nCommand '$1' not found, please install it first.\n\n$2\n"
        exit 1
    fi
}

check_cmd shadow "See https://shadow.github.io/docs/guide/install_shadow.html for installation, but use the \"ethereum\" branch from https://github.com/ppopth/shadow instead."
check_cmd geth "See https://geth.ethereum.org/docs/getting-started/installing-geth for more detail."
