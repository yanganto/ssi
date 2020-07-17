# SSI


[![Build Status](https://travis-ci.com/yanganto/ssi.svg?branch=master)](https://travis-ci.com/yanganto/ssi)

**S**ubstrate **S**torage **I**nspector is a tool to get the data in the DB of substrate base block chain.

## Usage

Given the `state root hash`, `storage key`, `rocks db path`, you can inspect the data in the DB which is used in the chain build based on Substrate.
The data will show in the node or in the nodes of a subtrie.


### Sample Commands

Here are examples help you can use in query the data of `Account` field in `System` pallet in Rocks DB at block #50

```
ssi -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -k 26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9 ./db
```

or 

```
ssi -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -F Account ./db -s
```


Also you can use following command to inspect the data of `System` pallet in Rocks DB at block #5

```
ssi -r 0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e -k 26aa394eea5630e07c48ae0c9558cef7 ./db
```

or

```
ssi -r 0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e -P System ./db
```

Besides, you can exactly insepc

### Options
Here is the required parameters to use this tool.
``` 
ssi --root-hash <root hash> --storage-key <storage key> <db path>
``` 

Following infomation is required:
- `db path`: the path to the rocksdb used to storage data of the chain build in Substrate
- `root hash`: the root hash for trie node in the chain build in Substrate
- storage key info, it can be provide by `storage key` or `pallet`, `field`, `twox 64 concat`, `black2 128 concat`, `twox 64 concat 2nd`, `black2 128 concat 2nd`
  - `storage key`: directly set the storage key used in substrate runtime
  - other options: these options are infomation from substrate pallet, and will be used to generate the storage key  

There are still some optional options to help you inspect the database.
- `-e`, exactly mode, this mode will no get the node in subtrie, only the data from the node exactly match the storage key.
- `-s`, sumary mode, this mode will show the data sumary of a node, if you only take care about data chaging without the exactly meaning.
- `-l <trace/debug/info/warn/error>`, show logs with different level
  - `info` level: the node type of storage key
  - `debug` level: the operation about path to trace the trie
  - `trace` level: the operation in DB layer for development
  - `warn` level: show the error if the database incorrect or damage or some node is incorrect
  - `error` level: show the error of this tool


### Snapshop

![snapshop](https://raw.githubusercontent.com/yanganto/ssi/master/demo.png)

## Solutions & How it works

```
|--------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| Layer        | The Info extracted from the layer                                                                                                                  |
| ==========   | ====================================================================================================================                               |
| Runtime      | -> Calculated: Hash("System") ++ Hahs("Account") ++ Hash(Account_ID) ++ Account_ID                                                                 |
|              | => 26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9 ++ Hash(Account_ID) ++ Account_ID                                              |
|              | * Need to read the definition in decl_stroage! of system pallet                                                                                    |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Backend      | -> Customized: RPC to get state root hash                                                                                                          |
|              | * Modify the source code of target chain to expose the state root hash                                                                             |
|--------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| Trie DB      | -> Calculateed node position base on following picture                                                                                             |
|              |                                                                                                                                                    |
|--------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| Rocks DB     | -> use state root hash -> get & decodde state root node -> get & decode childern -> get value                                                      |
|              |                                                                                                                                                    |
|--------------|----------------------------------------------------------------------------------------------------------------------------------------------------|

```

- ref: [decl stroage](https://github.com/paritytech/substrate/blob/master/frame/system/src/lib.rs#L414)of system pallet 

![snapshop](https://raw.githubusercontent.com/yanganto/ssi/master/trie.png)


## Important Reference
Before tracing, there are articles explaning the keys in substrate.
- https://www.shawntabrizi.com/substrate/transparent-keys-in-substrate/
- https://www.shawntabrizi.com/substrate/substrate-storage-deep-dive/

## Dependency Analysis

```
|---------------------------|--------------------------------------------------------------------------|------------------------------------|
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
The raw data get from the state root hash #50
128 (0x80) is the seperator and contain 6 hash
[
128,
44, 152,
128,
94, 160, 29, 150, 47, 206, 102, 184, 44, 189, 162, 219, 238, 99, 92, 175, 155, 113, 164, 89, 83, 126, 3, 28, 142, 179, 123, 67, 223, 54, 236, 110,
128,
254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20,
128,
186, 239, 159, 204, 255, 176, 65, 173, 46, 89, 234, 229, 224, 173, 99, 171, 160, 247, 64, 36, 224, 190, 171, 44, 238, 243, 103, 85, 219, 220, 88, 231,
128,
25, 0, 97, 20, 123, 121, 139, 175, 63, 105, 174, 89, 206, 29, 35, 156, 173, 24, 48, 50, 136, 242, 207, 8, 72, 46, 144, 52, 18, 133, 232, 112,
128,
129, 101, 66, 162, 246, 165, 217, 92, 249, 6, 252, 250, 25, 216, 48, 38, 83, 144, 124, 58, 177, 32, 205, 61, 138, 219, 219, 23, 15, 86, 157, 170,
128,
86, 19, 59, 211, 244,
128,
21, 23, 250, 142, 131, 23, 178, 43, 217, 200, 158, 181, 223, 190, 119, 217, 26, 9, 179, 225, 71, 97, 160, 254, 75, 108
]

Here is the decoded output, there are 6 child node have data
[None, None, Hash, Hash, None, Hash, None, None, None, None, None, Hash, Hash, None, None, Hash]
== NibbleBranch
== partial: NibbleSlicePlan { bytes: 1..1, offset: 0 }
== value: None
== child: Hash(4..36)    (32)  [85, 114, 90, 184, 81, 124, 23, 235, 182, 170, 184, 133, 223, 93, 124, 251, 173, 165, 190, 146, 234, 244, 217, 33, 222, 72, 35, 215, 116, 144, 19, 36]
== child: Hash(37..69)   (48)  [254, 64, 112, 194, 32, 25, 207, 40, 249, 31, 246, 143, 125, 54, 211, 81, 35, 61, 4, 132, 72, 140, 41, 106, 142, 111, 56, 156, 57, 89, 131, 20] (no change from #5 - #50)
== child: Hash(70..102)  (80)  [192, 98, 139, 161, 117, 76, 9, 117, 55, 27, 157, 185, 121, 240, 247, 152, 207, 145, 160, 222, 132, 194, 154, 32, 234, 200, 227, 208, 108, 197, 178, 7]
== child: Hash(103..135) (176) [49, 239, 154, 236, 197, 182, 244, 198, 173, 189, 166, 59, 187, 46, 204, 215, 78, 31, 59, 29, 95, 55, 237, 98, 113, 164, 48, 198, 16, 219, 132, 25]
== child: Hash(136..168) (192) [5, 186, 0, 144, 71, 86, 173, 65, 68, 101, 0, 237, 69, 84, 150, 46, 111, 109, 136, 206, 86, 88, 156, 177, 9, 20, 70, 100, 251, 208, 141, 121]
== child: Hash(169..201) (240) [160, 117, 69, 22, 65, 224, 115, 49, 4, 235, 92, 183, 242, 80, 47, 241, 125, 66, 246, 68, 19, 116, 151, 54, 185, 234, 44, 248, 13, 22, 45, 193]
```

### Possible Prefix
| Digtail | Hex | Meaning |
|---------|-----|---------|
| 32      | 20  |         |
| 48      | 30  |         |
| 80      | 50  |         |
| 176     | B0  |         |
| 192     | C0  |         |
| 240     | F0  |         |

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
The db is generated by [substrate-node-template](https://github.com/yanganto/substrate-node-template/tree/expose-ext-root) with dev chain option. 
- The root hashs in block #5
  - state root hash: "0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e"
  - extrinsics root hash: "0xc1f78e951f26fe2c55e10f32b7bc97a227ee59274fabff18e5eabb6bee70c494"
- The operation from block #5 to block #20
  - Alice transfers 128 to Bob
  - Bob transfers 64 to Charlie
  - Charlie transfers 32 to Dave
  - Dave transfers 16 to Eve
  - Eve transfer 8 to Ferdie
- The latest balance for peoples
  - Alice: 1,024.921
  - Alice Stash: 1,152.921
  - Bob: 1,216.921
  - Bob Stash: 1,152.921
  - Charlie: 31.999
  - Dave: 15.999
  - Eve: 7.999
  - Ferdie: 8.000
- The root hashs in block # 50
  - state root hash: "0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d"
  - extrinsics root hash: "0x2772dcca7b706ca5c9692cb02e373d667ab269ea9085eb55e6900584b7c2c682"
