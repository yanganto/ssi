# SSI

## Dependency Analysis

| Modules                   | Who use this                                                             | Important Dependency        |
|---------------------------|--------------------------------------------------------------------------|-----------------------------|
| frame/support             | frames                                                                   | sp-statemachine             |
|---------------------------|--------------------------------------------------------------------------|-----------------------------|
| primitives/state-manchine | frame/support                                                            | sp-core/storage(sp-storage) |
|---------------------------|--------------------------------------------------------------------------|-----------------------------|
| primitives/database       | client/db, client/api, utils/browser                                     | kvdb (parity-common)        |
|---------------------------|--------------------------------------------------------------------------|-----------------------------|
| primitives/storage        | client/service, client/api                                               | serde                       |
|                           | frame/democracy, frame/staking, frame/transcation-payment, frame/vesting |                             |
|                           | primitives/externalities, primitives/core                                |                             |
|                           | utils/frame/rpc/support                                                  |                             |

On Chain Side
- `client/api` -> `sp-core/storage` -> `primitives/storage`
- `frame/support` handle the **key** generation
- `sp-core/storage` export `sp-storage`
- `primitives/storage` handle the **storage key** generation

On Client Side
- `client/api` -> `primitives/database` -> `primitives/storage`
- `primitives/database` handle the **transcation** for key-value pair storage
- `primitives/storage` handle the **storage key** generation

## DB Sample
The db is generated by `substrate-node-template` at `93862bde52c77e045bdb6f60e9ba2269a6cad856` with dev chain option.

