#!/usr/bin/env node

const yargs = require('yargs');
const fs = require('fs');

const argv = yargs
    .option('address-file', {
        description: 'Deposit contract address file',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .option('config-file', {
        description: 'Lighthouse config file',
        type: 'string',
        demandOption: true,
        requiresArg: true,
    })
    .help()
    .alias('help', 'h').argv;

(async function() {
    const address = fs.readFileSync(argv.addressFile).toString();
    fs.appendFileSync(argv.configFile, "DEPOSIT_CONTRACT_ADDRESS: \"" + address + "\"");
    process.exit();
})();
