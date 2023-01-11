#!/usr/bin/env node

const yargs = require('yargs');
const fs = require('fs');
const Web3 = require('web3');

const argv = yargs
    .option('endpoint', {
        description: 'HTTP endpoint to which you want to deploy the deposit contract',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .option('file', {
        description: 'Deposit contract JSON file path',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .option('address-out', {
        description: 'The file path to write the contract address',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .option('block-out', {
        description: 'The file path to write the contract block number',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .help()
    .alias('file', 'f')
    .alias('help', 'h').argv;

(async function() {
    const web3 = new Web3(argv.endpoint);
    const accounts = await web3.eth.getAccounts();

    // The transaction in the mainnet is at
    // https://etherscan.io/tx/0xe75fb554e433e03763a1560646ee22dcb74e5274b34c5ad644e7c0f619a7e1d0
    const json = JSON.parse(fs.readFileSync(argv.file));
    const undeployed = new web3.eth.Contract(json.abi);
    // The real gasLimit is 3,141,592 and the real gasPrice is 147 Gwei
    const contract = await undeployed
        .deploy({ data: json.bytecode })
        .send({
            from: accounts[0],
            nonce: 0,
            gas: 3141592,
            gasPrice: '147000000000',
        })
        .once('transactionHash', hash => {
            console.log('transaction', hash);
        })
        .once('receipt', receipt => {
            console.log('block_number', receipt.blockNumber);
            fs.writeFileSync(argv.blockOut, receipt.blockNumber.toString());
        });

    console.log('address', contract.options.address);
    fs.writeFileSync(argv.addressOut, contract.options.address);
    process.exit();
})();
