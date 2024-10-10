# Supported Clients

✅ = Available, works out-of-the-box with latest release  
🚧 = Available, works with modifications (see subpage for details)  
❌ = Unavailable, does not currently work  
❔ = Unavailable, not yet tested

A client is considered to work if it can follow the chain and perform the necessary duties for validating. Other 
features might not work.

## Execution Layer

| Name                    | Node | Boot Node |
|-------------------------|:----:|:---------:|
| Besu                    |  ❔   |     ❔     |
| Erigon                  |  ❔   |     ❔     |
| EthereumJS              |  ❔   |     ❔     |
| [Geth](clients/geth.md) |  ✅   |     ✅     |
| Nethermind              |  ❔   |     ❔     |
| [Reth](clients/reth.md) |  🚧  |     ❔     |


## Consensus Layer

| Name                                | Node | Boot Node | Validator Client |
|-------------------------------------|:----:|:---------:|:----------------:|
| Grandine                            |  ❔   |     ❔     |        ❔         |
| [Lighthouse](clients/lighthouse.md) |  ✅   |     ✅     |        ✅         |
| Lodestar                            |  ❔   |     ❔     |        ❔         |
| Nimbus                              |  ❔   |     ❔     |        ❔         |
| Prysm                               |  ❔   |     ❔     |        ❔         |
| Teku                                |  ❔   |     ❔     |        ❔         |

## Other

| Name                                | Status | Description                                                                             |
|-------------------------------------|:------:|-----------------------------------------------------------------------------------------|
| [Blobssss](clients/blobssss.md)     |   ✅    | Simple blob transaction spammer designed for use in `ethshadow`                         |
| [Prometheus](clients/prometheus.md) |   ✅    | Used to capture metrics provided by the clients, currently only Lighthouse is supported |

