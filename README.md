# SSI
**S**ubstrate **S**torage **I**nspector is a tool to get the data in the DB of substrate base block chain.

## Usage

:construction: :construction: :construction: Under development and nothing usable :construction: :construction: :construction:

## Important Reference
Before tracing, there are articles explaning the keys in substrate.
- https://www.shawntabrizi.com/substrate/transparent-keys-in-substrate/
- https://www.shawntabrizi.com/substrate/substrate-storage-deep-dive/

## Dependency Analysis

```
| Modules                   | Who use this                                                             | Important Dependency               |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| frame/support             | frames                                                                   | sp-statemachine                    |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/state-manchine | frame/support                                                            | sp-core/storage(sp-storage),       |
|                           |                                                                          | trie-db(parity), trie-root(parity) |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/database       | client/db, client/api, utils/browser                                     | kvdb (parity-common)               |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/storage        | client/service, client/api                                               | serde                              |
|                           | frame/democracy, frame/staking, frame/transcation-payment, frame/vesting |                                    |
|                           | primitives/externalities, primitives/core                                |                                    |
|                           | utils/frame/rpc/support                                                  |                                    |
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
| primitives/trie           | client/service, client/api, client/executor, client/block-builder        |                                    |
|                           | frame/session                                                            |                                    |
|                           | primitives/io, primitives/state-machine                                  |                                    |
```


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
    ```rust
      impl<D: Borrow<[u8]>> OwnedNode<D> {
        /// Construct an `OwnedNode` by decoding an owned data source according to some codec.
        pub fn new<C: NodeCodec>(data: D) -> Result<Self, C::Error> {
          let plan = C::decode_plan(data.borrow())?;
          Ok(OwnedNode { data, plan })
        }

        /// Returns a reference to the backing data.
        pub fn data(&self) -> &[u8] {
          self.data.borrow()
        }

        /// Returns a reference to the node decode plan.
        pub fn node_plan(&self) -> &NodePlan {
          &self.plan
        }

        /// Construct a `Node` by borrowing data from this struct.
        pub fn node(&self) -> Node {
          self.plan.build(self.data.borrow())
        }
      }
    ```
    ```rust
    /// A reference to a trie node which may be stored within another trie node.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NodeHandle<'a> {
      Hash(&'a [u8]),
      Inline(&'a [u8]),
    }
    ```
  - `trie-root` - a root calculated entirely in-memory

### hash of trie 
  - the hash root of extrinsic can be caluculate by `extrinsics_root` and `state_root` in system frame
  - the state trie root hash and extrinsics root hash are in block header
  - the block header is handled in the client modules
  - the data structure use `0x80` as a seperator, which is invisible because it is not in ISO-IEC 5559 (ASCII) and not in ISO-IEC 5559-1 (EASCII) 

```
The raw data get from the state root hash
128 (0x80) is the seperator and contain 6 hash
[
128,
44, 152,
128,
195, 162, 207, 123, 55, 18, 222, 77, 1, 22, 17, 199, 219, 213, 251, 6, 219, 238, 93, 205, 228, 57, 200, 241, 174, 67, 28, 169, 56, 67, 133, 227,
128,
163, 1, 165, 139, 133, 248, 193, 151, 249, 13, 162, 27, 86, 16, 212, 200, 254, 39, 224, 160, 190, 105, 225, 221, 19, 7, 246, 109, 6, 202, 107, 194,
128,
183, 207, 190, 0, 154, 221, 23, 122, 61, 23, 200, 77, 4, 206, 177, 67, 75, 70, 146, 247, 160, 60, 44, 1, 193, 112, 28, 200, 207, 30, 252, 13,
128,
169, 211, 94, 35, 16, 145, 205, 137, 159, 42, 39, 155, 8, 205, 23, 49, 194, 6, 0, 48, 123, 252, 28, 183, 111, 148, 14, 163, 184, 197, 136, 197,
128,
129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170,
128,
14, 118, 239, 35, 99, 149, 106, 122, 22, 52, 150, 153, 107, 252, 84, 152, 196, 146, 67, 247, 231, 69, 60, 230, 252, 236, 175, 161, 113, 48, 23, 153
]

Here is the decoded output, there are 6 child node have data
[None, None, Hash, Hash, None, Hash, None, None, None, None, None, Hash, Hash, None, None, Hash]
== child: Hash(4..36)       Hash([195, 162, 207, 123, 55, 18, 222, 77, 1, 22, 17, 199, 219, 213, 251, 6, 219, 238, 93, 205, 228, 57, 200, 241, 174, 67, 28, 169, 56, 67, 133, 227]),
== child: Hash(37..69)      Hash([163, 1, 165, 139, 133, 248, 193, 151, 249, 13, 162, 27, 86, 16, 212, 200, 254, 39, 224, 160, 190, 105, 225, 221, 19, 7, 246, 109, 6, 202, 107, 194])
== child: Hash(70..102)     Hash([183, 207, 190, 0, 154, 221, 23, 122, 61, 23, 200, 77, 4, 206, 177, 67, 75, 70, 146, 247, 160, 60, 44, 1, 193, 112, 28, 200, 207, 30, 252, 13])
== child: Hash(103..135)    Hash([169, 211, 94, 35, 16, 145, 205, 137, 159, 42, 39, 155, 8, 205, 23, 49, 194, 6, 0, 48, 123, 252, 28, 183, 111, 148, 14, 163, 184, 197, 136, 197])
== child: Hash(136..168)    Hash([129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170])
== child: Hash(169..201)    Hash([14, 118, 239, 35, 99, 149, 106, 122, 22, 52, 150, 153, 107, 252, 84, 152, 196, 146, 67, 247, 231, 69, 60, 230, 252, 236, 175, 161, 113, 48, 23, 153])
```

### Conclusion
To get things in the db and by pass the RPC, following parameters are needed:
- state trie root hash, extrinsics root hash
- NibbleSlice struct, which is maded from key

## RPC Calls for Root Hash
The reading the DB and inspect the data without less substrate dependency may be possible.  
As aforementioned, the root hash is needed, and can be exposed by RPC.

Here is the [customized substrate node-template](https://github.com/yanganto/substrate-node-template/tree/expose-ext-root) with two RPC calls, `state_queryExtrinsicsRoot` and `state_queryStateRoot`, to get the root hash of current block or specific blocks.
Following is the RPC call with curl command to help you get the root hash.
- ```curl -X POST -H "Content-Type: application/json"  --data '{"id":6,"jsonrpc":"2.0","method":"state_queryStateRoot","params":[]}' 127.0.0.1:9933```
- ```curl -X POST -H "Content-Type: application/json"  --data '{"id":6,"jsonrpc":"2.0","method":"state_queryExtrinsicsRoot","params":[]}' 127.0.0.1:9933```

## Storage Parameters in RockDB
### Column Family
The state root hash is store as a key in Column Family `col1`and the column family meanings are shown as following source code in Substrate.

```rust
pub(crate) mod columns {
	pub const META: u32 = crate::utils::COLUMN_META;
	pub const STATE: u32 = 1;
	pub const STATE_META: u32 = 2;
	/// maps hashes to lookup keys and numbers to canon hashes.
	pub const KEY_LOOKUP: u32 = 3;
	pub const HEADER: u32 = 4;
	pub const BODY: u32 = 5;
	pub const JUSTIFICATION: u32 = 6;
	pub const CHANGES_TRIE: u32 = 7;
	pub const AUX: u32 = 8;
	/// Offchain workers local storage
	pub const OFFCHAIN: u32 = 9;
	pub const CACHE: u32 = 10;
}
```

### DB Sample
The db is generated by [substrate-node-template](https://github.com/yanganto/substrate-node-template/tree/expose-ext-root) with dev chain option. The state root hash is "0x09c0a468b841682c4cf29297408fadba23329fb7c0c5c81163c40f28f5caa5cd", and the extrinsics root hash is "0xb8946898950fe86044251bf4b9e0a71c0304d119fdee180ab6129059c698dbd1".

