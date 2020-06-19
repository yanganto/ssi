# SSI

# Important Reference
Before tracing, there are articles explaning the keys in substrate.
- https://www.shawntabrizi.com/substrate/transparent-keys-in-substrate/
- https://www.shawntabrizi.com/substrate/substrate-storage-deep-dive/

## Dependency Analysis

| Modules                   | Who use this                                                             | Important Dependency               |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| frame/support             | frames                                                                   | sp-statemachine                    |

| Modules                   | Who use this                                                             | Important Dependency               |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/state-manchine | frame/support                                                            | sp-core/storage(sp-storage),       |
|                           |                                                                          | trie-db(parity), trie-root(parity) |

| Modules                   | Who use this                                                             | Important Dependency               |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/database       | client/db, client/api, utils/browser                                     | kvdb (parity-common)               |

| Modules                   | Who use this                                                             | Important Dependency               |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/storage        | client/service, client/api                                               | serde                              |
|                           | frame/democracy, frame/staking, frame/transcation-payment, frame/vesting |                                    |
|                           | primitives/externalities, primitives/core                                |                                    |
|                           | utils/frame/rpc/support                                                  |                                    |

| Modules                   | Who use this                                                             | Important Dependency               |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/trie           | client/service, client/api, client/executor, client/block-builder        |                                    |
|                           | frame/session                                                            |                                    |
|                           | primitives/io, primitives/state-machine                                  |                                    |


### Storage path
- On chain side: `frames` -> `frame/support` -> `sp-statemachine` -> `primitives/storage`, `trie-db`, `trie-root`
- On client side: `client/api` -> `primitives/database` -> `kvdb`
- `frame/support` handle the **key** generation
  - `procedural/src/lib.rs` - procedure macro `decl_storage`

- `sp-core/storage` export as `sp-storage`, handle the **storage key** generation
- `primitives/database` handle the **transcation** for key-value pair storage
- `primitives/storage` handle the **storage key** generation
- `primitives/state-machine` stores things into trie
  - [`fn storage`](https://github.com/paritytech/substrate/blob/master/primitives/state-machine/src/backend.rs#L44)
  - `proving_backend.rs` [ProvingBackendRecorder](https://github.com/paritytech/substrate/blob/master/primitives/state-machine/src/proving_backend.rs#L36)
  - `trie_backend_essence.rs` [TrieBackendEssence](https://github.com/paritytech/substrate/blob/master/primitives/state-machine/src/trie_backend_essence.rs#L40)
    - [`fn storage`](https://github.com/paritytech/substrate/blob/master/primitives/state-machine/src/trie_backend_essence.rs#L161)
      - This function need "trie root" and "key" to get the value
- `primitives/trie` - a warapper to use trie-db, this is the last module in `substrate` repo.
  - [`fn read_trie_value`](https://github.com/paritytech/substrate/blob/master/primitives/trie/src/lib.rs#L189)
- `trie-db`, `trie-root` are included by `Trie`
  - `trie-db` - a backend database to provide a persistent trie structure (Merkle-Patricia Trie)
    - [`fn new`](https://github.com/paritytech/trie/blob/master/trie-db/src/triedb.rs#L66) for TrieDB
    - [`fn get`](https://github.com/paritytech/trie/blob/master/trie-db/src/lib.rs#L196)- key -> Result<Option<DBValue>, TrieHash<L>, CError<L>>
    - [`fn get_with`](https://github.com/paritytech/trie/blob/master/trie-db/src/lib.rs#L205) with [`Query Trait`](https://github.com/paritytech/trie/blob/master/trie-db/src/lib.rs#L150)
    - [`fn get_with`](https://github.com/paritytech/trie/blob/master/trie-db/src/triedb.rs#L124) implementation for triedb 
    - trie-db query with `NibbleSlice` by [`Lookup` trait](https://github.com/paritytech/trie/blob/master/trie-db/src/lookup.rs#L41)

    ```rust
      pub type Result<T, H, E> = crate::rstd::result::Result<T, Box<TrieError<H, E>>>;

      pub fn look_up(mut self, key: NibbleSlice) -> Result<Option<Q::Item>, TrieHash<L>, CError<L>>

      /// - db.get(hash, Prefix)
      ///     - Prefix is compelet part and nibble part

      for  depth in 0 .. {
        let node_data = match self.db.get(&hash, key.mid(key_nibbles).left());
        loop {
          let decoded = match L::Codec::decode(node_data);
          let next_node = match decoded {
            NodeMatched(slice, value) => {
              return (self.query.decode(value))
            }
          }
        }
      }

    ```

    - [`to_storeed`] of `NibbleSlice` reutrn NodeKey (usize, nibble::BackingByteVec) (the key removed the parts from parrents)

    ```rust
      enum Node<H> {
        /// Empty node.
        Empty,
        /// A leaf node contains the end of a key and a value.
        /// This key is encoded from a `NibbleSlice`, meaning it contains
        /// a flag indicating it is a leaf.
        Leaf(NodeKey, DBValue),
        /// An extension contains a shared portion of a key and a child node.
        /// The shared portion is encoded from a `NibbleSlice` meaning it contains
        /// a flag indicating it is an extension.
        /// The child node is always a branch.
        Extension(NodeKey, NodeHandle<H>),
        /// A branch has up to 16 children and an optional value.
        Branch(Box<[Option<NodeHandle<H>>; 16]>, Option<DBValue>),
        /// Branch node with support for a nibble (to avoid extension node).
        NibbledBranch(NodeKey, Box<[Option<NodeHandle<H>>; 16]>, Option<DBValue>),
      }
    ```

  - `trie-root` - a root calculated entirely in-memory

### hash of trie 
  - the hash root of extrinsic can be caluculate by `extrinsics_root` and `extrinsics_data_root` in system frame
  - the state trie root hash and extrinsics root hash are in block header
  - the block header is handled in the client modules

### Conclusion
To get things in the db and by pass the RPC, following parameters are needed:
- state trie root hash, extrinsics root hash
- NibbleSlice struct, which is maded from key

## DB Sample
The db is generated by `substrate-node-template` at `93862bde52c77e045bdb6f60e9ba2269a6cad856` with dev chain option.

