source ./vars.env

el_data_dir() {
    el_data_dir="$ROOT/node$1/ethereum"
}

cl_data_dir() {
    cl_data_dir="$ROOT/node$1/lighthouse"
}

node_ip() {
    ip=$(./utils/dec2ip.sh $(expr $(./utils/ip2dec.sh $BASE_NODE_IP) + $1))
}
